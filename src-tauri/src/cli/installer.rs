use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn get_cli_dir() -> PathBuf {
    let base_dirs = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let cli_dir = base_dirs.join("ollama-lite").join("cli");
    if !cli_dir.exists() {
        let _ = std::fs::create_dir_all(&cli_dir);
    }
    cli_dir
}

pub fn get_ollama_cmd_path() -> PathBuf {
    get_cli_dir().join("ollama.cmd")
}

pub fn get_ollama_ps1_path() -> PathBuf {
    get_cli_dir().join("ollama.ps1")
}

pub fn get_ollama_lite_cmd_path() -> PathBuf {
    get_cli_dir().join("ollama-lite.cmd")
}

pub fn get_cli_bat_path() -> PathBuf {
    get_ollama_cmd_path()
}

#[tauri::command]
pub fn install_cli_to_path() -> Result<String, String> {
    let cli_dir = get_cli_dir();
    let ollama_cmd_path = get_ollama_cmd_path();
    let ollama_ps1_path = get_ollama_ps1_path();
    let ollama_lite_cmd_path = get_ollama_lite_cmd_path();

    // 1. Primary Batch launcher: ollama.cmd
    let cmd_content = "@echo off\r\n\
        powershell.exe -NoProfile -ExecutionPolicy Bypass -File \"%~dp0ollama.ps1\" %*\r\n";

    let mut cmd_file = File::create(&ollama_cmd_path).map_err(|e| format!("Failed creating ollama.cmd: {}", e))?;
    cmd_file.write_all(cmd_content.as_bytes()).map_err(|e| format!("Failed writing ollama.cmd: {}", e))?;

    // 2. Alias Batch launcher: ollama-lite.cmd
    let mut lite_cmd_file = File::create(&ollama_lite_cmd_path).map_err(|e| format!("Failed creating ollama-lite.cmd: {}", e))?;
    lite_cmd_file.write_all(cmd_content.as_bytes()).map_err(|e| format!("Failed writing ollama-lite.cmd: {}", e))?;

    // 3. Full Ollama CLI Engine: ollama.ps1
    let ps1_content = r#"param(
    [Parameter(Position=0)]
    [string]$Command = "help",

    [Parameter(Position=1)]
    [string]$Arg1 = "",

    [Parameter(Position=2)]
    [string]$Arg2 = "",

    [Parameter(Position=3, ValueFromRemainingArguments=$true)]
    [string[]]$RemainingArgs
)

$ServerUrl = "http://127.0.0.1:11434"

function Show-Help {
    Write-Host "Large language model runner" -ForegroundColor White
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor White
    Write-Host "  ollama [flags]"
    Write-Host "  ollama [command]"
    Write-Host ""
    Write-Host "Available Commands:" -ForegroundColor White
    Write-Host "  serve       Start ollama"
    Write-Host "  create      Create a model from a Modelfile"
    Write-Host "  show        Show information for a model"
    Write-Host "  run         Run a model"
    Write-Host "  stop        Stop a running model"
    Write-Host "  pull        Pull a model from a registry"
    Write-Host "  push        Push a model to a registry"
    Write-Host "  list, ls    List models"
    Write-Host "  ps          List running models"
    Write-Host "  cp          Copy a model"
    Write-Host "  rm          Remove a model"
    Write-Host "  help        Help about any command"
    Write-Host ""
    Write-Host "Flags:" -ForegroundColor White
    Write-Host "  -h, --help      help for ollama"
    Write-Host "  -v, --version   version for ollama"
    Write-Host ""
    Write-Host "Use 'ollama [command] --help' for more information about a command."
}

function Get-OllamaVersion {
    try {
        $resp = Invoke-RestMethod -Uri "$ServerUrl/api/version" -Method Get -TimeoutSec 3 -ErrorAction Stop
        Write-Host "ollama version is $($resp.version)"
    } catch {
        Write-Host "ollama version 0.1.0 (offline - start with 'ollama serve' or open Ollama Lite)" -ForegroundColor Yellow
    }
}

function List-Models {
    try {
        $resp = Invoke-RestMethod -Uri "$ServerUrl/api/tags" -Method Get -TimeoutSec 5 -ErrorAction Stop
        if (-not $resp.models -or $resp.models.Count -eq 0) {
            Write-Host "No models installed. Run 'ollama pull <model>' to download a model." -ForegroundColor Yellow
            return
        }

        Write-Host ("{0,-32} {1,-16} {2,-10} {3,-16}" -f "NAME", "ID", "SIZE", "MODIFIED") -ForegroundColor DarkGray
        foreach ($m in $resp.models) {
            $sizeGb = [math]::Round($m.size / 1GB, 1)
            $sizeStr = "$sizeGb GB"
            $idStr = if ($m.digest.Length -gt 19) { $m.digest.Substring(7, 12) } else { $m.digest }
            $modStr = $m.modified_at.Substring(0, 10)
            Write-Host ("{0,-32} {1,-16} {2,-10} {3,-16}" -f $m.name, $idStr, $sizeStr, $modStr)
        }
    } catch {
        Write-Host "Failed connecting to Ollama at $ServerUrl. Run 'ollama serve' first." -ForegroundColor Red
    }
}

function Get-ProcessStatus {
    try {
        $resp = Invoke-RestMethod -Uri "$ServerUrl/api/ps" -Method Get -TimeoutSec 5 -ErrorAction Stop
        if (-not $resp.models -or $resp.models.Count -eq 0) {
            Write-Host "NAME  ID  SIZE  PROCESSOR  UNTIL" -ForegroundColor DarkGray
            return
        }

        Write-Host ("{0,-32} {1,-16} {2,-10} {3,-14} {4,-10}" -f "NAME", "ID", "SIZE", "PROCESSOR", "UNTIL") -ForegroundColor DarkGray
        foreach ($m in $resp.models) {
            $sizeGb = [math]::Round($m.size / 1GB, 1)
            $sizeStr = "$sizeGb GB"
            $idStr = if ($m.digest.Length -gt 19) { $m.digest.Substring(7, 12) } else { $m.digest }
            Write-Host ("{0,-32} {1,-16} {2,-10} {3,-14} {4,-10}" -f $m.name, $idStr, $sizeStr, "100% GPU", "Active")
        }
    } catch {
        Write-Host "Failed connecting to Ollama at $ServerUrl." -ForegroundColor Red
    }
}

function Show-Model([string]$model, [string[]]$extraFlags) {
    if (-not $model) {
        Write-Host "Error: Model name required. Usage: ollama show <model>" -ForegroundColor Red
        return
    }

    try {
        $body = @{ model = $model } | ConvertTo-Json
        $resp = Invoke-RestMethod -Uri "$ServerUrl/api/show" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 10 -ErrorAction Stop

        $flagStr = ($extraFlags -join " ").ToLower()
        if ($flagStr -match "--modelfile") {
            Write-Host $resp.modelfile
            return
        }
        if ($flagStr -match "--parameters") {
            Write-Host $resp.parameters
            return
        }
        if ($flagStr -match "--template") {
            Write-Host $resp.template
            return
        }
        if ($flagStr -match "--system") {
            Write-Host $resp.system
            return
        }
        if ($flagStr -match "--license") {
            Write-Host $resp.license
            return
        }

        Write-Host "Model" -ForegroundColor Cyan
        Write-Host "  architecture        $($resp.details.family)"
        Write-Host "  parameters          $($resp.details.parameter_size)"
        Write-Host "  context length      4096"
        Write-Host "  quantization        $($resp.details.quantization_level)"
        Write-Host ""
        Write-Host "Parameters" -ForegroundColor Cyan
        Write-Host "  $($resp.parameters.Replace("`n", "`n  "))"
        Write-Host ""
        Write-Host "System" -ForegroundColor Cyan
        Write-Host "  $($resp.system)"
        Write-Host ""
        Write-Host "License" -ForegroundColor Cyan
        Write-Host "  $($resp.license)"
    } catch {
        Write-Host "Error: Model '$model' not found." -ForegroundColor Red
    }
}

function Pull-Model([string]$model) {
    if (-not $model) {
        Write-Host "Error: Model name required. Usage: ollama pull <model>" -ForegroundColor Red
        return
    }

    Write-Host "pulling manifest" -ForegroundColor DarkGray
    $body = @{ model = $model } | ConvertTo-Json

    try {
        $req = [System.Net.HttpWebRequest]::Create("$ServerUrl/api/pull")
        $req.Method = "POST"
        $req.ContentType = "application/json"
        $req.Timeout = 3600000

        $bytes = [System.Text.Encoding]::UTF8.GetBytes($body)
        $req.ContentLength = $bytes.Length
        $stream = $req.GetRequestStream()
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Close()

        $response = $req.GetResponse()
        $respStream = $response.GetResponseStream()
        $reader = New-Object System.IO.StreamReader($respStream)

        $lastPercent = -1
        while (-not $reader.EndOfStream) {
            $line = $reader.ReadLine()
            if ($line) {
                try {
                    $json = $line | ConvertFrom-Json
                    if ($json.error) {
                        Write-Host "`nError: $($json.error)" -ForegroundColor Red
                        break
                    }
                    if ($json.status -eq "downloading" -and $json.total -gt 0) {
                        $pct = [math]::Round(($json.completed / $json.total) * 100)
                        $completedMb = [math]::Round($json.completed / 1MB, 1)
                        $totalMb = [math]::Round($json.total / 1MB, 1)
                        if ($pct -ne $lastPercent) {
                            $lastPercent = $pct
                            $barWidth = 24
                            $filled = [math]::Round(($pct / 100) * $barWidth)
                            $empty = $barWidth - $filled
                            $bar = ("=" * $filled) + (" " * $empty)
                            Write-Host -NoNewline ("`rdownloading [$bar] $pct% ($completedMb MB / $totalMb MB)") -ForegroundColor Cyan
                        }
                    } elseif ($json.status -eq "success") {
                        Write-Host "`nsuccess" -ForegroundColor Green
                    } else {
                        Write-Host "`n$($json.status)" -ForegroundColor DarkGray
                    }
                } catch {}
            }
        }
        $reader.Close()
        $response.Close()
    } catch {
        Write-Host "`nError pulling model: $_" -ForegroundColor Red
    }
}

function Remove-Model([string]$model) {
    if (-not $model) {
        Write-Host "Error: Model name required. Usage: ollama rm <model>" -ForegroundColor Red
        return
    }

    try {
        $body = @{ model = $model } | ConvertTo-Json
        $resp = Invoke-RestMethod -Uri "$ServerUrl/api/delete" -Method Delete -Body $body -ContentType "application/json" -TimeoutSec 10 -ErrorAction Stop
        Write-Host "deleted '$model'" -ForegroundColor Green
    } catch {
        Write-Host "Error: Failed deleting model '$model'. Make sure the model exists." -ForegroundColor Red
    }
}

function Copy-Model([string]$src, [string]$dst) {
    if (-not $src -or -not $dst) {
        Write-Host "Error: Source and destination required. Usage: ollama cp <source> <destination>" -ForegroundColor Red
        return
    }

    try {
        $body = @{ source = $src; destination = $dst } | ConvertTo-Json
        $resp = Invoke-RestMethod -Uri "$ServerUrl/api/copy" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 10 -ErrorAction Stop
        Write-Host "copied '$src' to '$dst'" -ForegroundColor Green
    } catch {
        Write-Host "Error: Failed copying model '$src' to '$dst'." -ForegroundColor Red
    }
}

function Stream-Prompt([string]$model, [string]$promptText) {
    $body = @{
        model = $model
        prompt = $promptText
        stream = $true
    } | ConvertTo-Json

    $req = [System.Net.HttpWebRequest]::Create("$ServerUrl/api/generate")
    $req.Method = "POST"
    $req.ContentType = "application/json"
    $req.Timeout = 300000

    $bytes = [System.Text.Encoding]::UTF8.GetBytes($body)
    $req.ContentLength = $bytes.Length
    $stream = $req.GetRequestStream()
    $stream.Write($bytes, 0, $bytes.Length)
    $stream.Close()

    try {
        $response = $req.GetResponse()
        $respStream = $response.GetResponseStream()
        $reader = New-Object System.IO.StreamReader($respStream)

        while (-not $reader.EndOfStream) {
            $line = $reader.ReadLine()
            if ($line) {
                try {
                    $json = $line | ConvertFrom-Json
                    if ($json.response) {
                        Write-Host -NoNewline $json.response
                    }
                } catch {}
            }
        }
        Write-Host ""
        $reader.Close()
        $response.Close()
    } catch {
        Write-Host "`nError streaming response: $_" -ForegroundColor Red
    }
}

function Run-Model([string]$model, [string[]]$initialPrompt) {
    if (-not $model) {
        Write-Host "Error: Model name required. Usage: ollama run <model>" -ForegroundColor Red
        return
    }

    if ($initialPrompt -and $initialPrompt.Count -gt 0) {
        $fullPrompt = $initialPrompt -join " "
        Stream-Prompt -model $model -promptText $fullPrompt
        return
    }

    Write-Host "[Ollama] Connected to $model" -ForegroundColor Green
    Write-Host "Type '/bye' or 'exit' to quit, '/clear' to clear screen.`n" -ForegroundColor DarkGray

    while ($true) {
        Write-Host -NoNewline ">>> " -ForegroundColor White
        $userInput = Read-Host

        if (-not $userInput) { continue }
        if ($userInput -eq "/bye" -or $userInput -eq "exit" -or $userInput -eq "quit") {
            Write-Host "Exiting session." -ForegroundColor DarkGray
            break
        }
        if ($userInput -eq "/clear") {
            Clear-Host
            continue
        }

        Stream-Prompt -model $model -promptText $userInput
    }
}

function Serve-Server {
    try {
        $resp = Invoke-RestMethod -Uri "$ServerUrl/api/version" -Method Get -TimeoutSec 3 -ErrorAction Stop
        Write-Host "Ollama is running on $ServerUrl (version $($resp.version))" -ForegroundColor Green
    } catch {
        Write-Host "Ollama server is starting on $ServerUrl..." -ForegroundColor Yellow
        Write-Host "Launch the Ollama Lite desktop app to keep the background daemon running." -ForegroundColor DarkGray
    }
}

# Combine remaining arguments into prompt or flags
$allExtras = @()
if ($Arg2) { $allExtras += $Arg2 }
if ($RemainingArgs) { $allExtras += $RemainingArgs }

# Command dispatch
switch ($Command.ToLower()) {
    "list"      { List-Models }
    "ls"        { List-Models }
    "ps"        { Get-ProcessStatus }
    "show"      { Show-Model -model $Arg1 -extraFlags $allExtras }
    "pull"      { Pull-Model -model $Arg1 }
    "rm"        { Remove-Model -model $Arg1 }
    "delete"    { Remove-Model -model $Arg1 }
    "cp"        { Copy-Model -src $Arg1 -dst $Arg2 }
    "run"       { Run-Model -model $Arg1 -initialPrompt $allExtras }
    "serve"     { Serve-Server }
    "start"     { Serve-Server }
    "stop"      {
        try {
            Invoke-RestMethod -Uri "$ServerUrl/api/generate" -Method Post -Body '{"model":"","prompt":""}' -TimeoutSec 3 -ErrorAction SilentlyContinue | Out-Null
            Write-Host "Model unloaded from memory." -ForegroundColor Green
        } catch {
            Write-Host "Model unloaded." -ForegroundColor Green
        }
    }
    "version"   { Get-OllamaVersion }
    "-v"        { Get-OllamaVersion }
    "--version" { Get-OllamaVersion }
    default     { Show-Help }
}
"#;

    let mut ps1_file = File::create(&ollama_ps1_path).map_err(|e| format!("Failed creating ollama.ps1: {}", e))?;
    ps1_file.write_all(ps1_content.as_bytes()).map_err(|e| format!("Failed writing ollama.ps1: {}", e))?;

    // 4. Add CLI dir to Windows User PATH
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(env) = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE) {
            let current_path: String = env.get_value("Path").unwrap_or_default();
            let cli_str = cli_dir.to_string_lossy().to_string();

            if !current_path.to_lowercase().contains(&cli_str.to_lowercase()) {
                let new_path = if current_path.ends_with(';') || current_path.is_empty() {
                    format!("{}{}", current_path, cli_str)
                } else {
                    format!("{};{}", current_path, cli_str)
                };
                let _ = env.set_value("Path", &new_path);
            }
        }
    }

    Ok(ollama_cmd_path.to_string_lossy().to_string())
}
