# Use este script para executar testes locais

$RESULTS_WORKSPACE = "$(Get-Location)\load-test\user-files\results"
$GATLING_WORKSPACE = "$(Get-Location)\load-test\user-files"

function Run-Gatling {
    & "$env:GATLING_HOME/bin\gatling.bat" -rm local -s RinhaBackendCrebitosSimulation `
        -rd "Rinha de Backend - 2024/Q1: Crébito" `
        -rf $RESULTS_WORKSPACE `
        -sf "$GATLING_WORKSPACE/simulations"
}

function Start-Test {
    for ($i = 1; $i -le 20; $i++) {
        Write-Output $i
        try {
            # 2 requests to wake the 2 API instances up :)
            Invoke-RestMethod -Uri "http://localhost:9999/clientes/1/extrato" -ErrorAction Stop
            Write-Host ""
            Invoke-RestMethod -Uri "http://localhost:9999/clientes/1/extrato" -ErrorAction Stop
            Write-Host ""
            Run-Gatling
            break
        } catch {
            Write-Host "API is not ready yet. Retrying in 2 seconds..."
            Write-Host "Encountered Error:"$_.Exception.Message
            Start-Sleep -Seconds 2
        }
    }
}

Start-Test
