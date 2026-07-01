param(
    [string[]]$Languages = @("typescript"),
    [string]$BaseUrl = "http://localhost:18095",
    [string]$SdkVersion = "0.1.0",
    [ValidateSet("app", "backend")]
    [string]$SdkType = "backend"
)

$ErrorActionPreference = "Stop"

function Resolve-PackageName {
    param([string]$Language, [string]$Type)

    if ($Type -eq "backend") {
        switch ($Language) {
            "typescript" { return "@sdkwork/account-backend-sdk" }
            default { return "sdkwork-account-backend-sdk-$Language" }
        }
    }

    switch ($Language) {
        "typescript" { return "@sdkwork/account-app-sdk" }
        default { return "sdkwork-account-app-sdk-$Language" }
    }
}

function Resolve-ClientName {
    param([string]$Type)
    if ($Type -eq "backend") { return "SdkworkAccountBackendClient" }
    return "SdkworkAccountAppClient"
}

function Resolve-ApiPrefix {
    param([string]$Type)
    if ($Type -eq "backend") { return "/backend/v3/api" }
    return "/app/v3/api"
}

function Resolve-OpenApiInput {
    param([string]$FamilyRoot, [string]$Type)
    if ($Type -eq "backend") {
        return Join-Path $FamilyRoot "openapi\account-backend-api.openapi.json"
    }
    return Join-Path $FamilyRoot "openapi\account-app-api.openapi.json"
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$FamilyRoot = (Get-Item $ScriptDir).Parent.FullName
$AccountRoot = (Get-Item (Join-Path $FamilyRoot "..\..")).FullName
$WorkspaceRoot = (Get-Item (Join-Path $AccountRoot "..")).FullName
$GeneratorPath = Join-Path $WorkspaceRoot "sdkwork-sdk-generator\bin\sdkgen.js"
$SdkName = Split-Path -Leaf $FamilyRoot
$InputPath = Resolve-OpenApiInput $FamilyRoot $SdkType
$ApiPrefix = Resolve-ApiPrefix $SdkType
$ClientName = Resolve-ClientName $SdkType

& node (Join-Path $AccountRoot "scripts\materialize-account-openapi-v3-security.mjs") | Out-Host

if (-not (Test-Path $GeneratorPath)) {
    throw "Canonical SDK generator not found: $GeneratorPath"
}
if (-not (Test-Path $InputPath)) {
    throw "OpenAPI input not found: $InputPath"
}

foreach ($LanguageValue in $Languages) {
    foreach ($LanguagePart in "$LanguageValue".Split(",")) {
        $Language = $LanguagePart.Trim()
        if ([string]::IsNullOrWhiteSpace($Language)) {
            continue
        }

        $LanguageWorkspace = Join-Path $FamilyRoot "$SdkName-$Language"
        $OutputPath = Join-Path $LanguageWorkspace "generated\server-openapi"
        $PackageName = Resolve-PackageName $Language $SdkType
        $ResolvedLanguageWorkspace = [System.IO.Path]::GetFullPath($LanguageWorkspace)
        $ResolvedOutputPath = [System.IO.Path]::GetFullPath($OutputPath)
        $LanguageWorkspacePrefix = $ResolvedLanguageWorkspace.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar

        if (-not $ResolvedOutputPath.StartsWith($LanguageWorkspacePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to clean SDK output outside language workspace: $ResolvedOutputPath"
        }

        if (Test-Path $OutputPath) {
            Remove-Item -LiteralPath $OutputPath -Recurse -Force
        }

        Write-Host "Generating $Language SDK at $OutputPath" -ForegroundColor Cyan
        & node $GeneratorPath generate `
            -i $InputPath `
            -o $OutputPath `
            -n $SdkName `
            -t $SdkType `
            -l $Language `
            --fixed-sdk-version $SdkVersion `
            --base-url $BaseUrl `
            --api-prefix $ApiPrefix `
            --package-name $PackageName `
            --client-name $ClientName `
            --standard-profile sdkwork-v3 `
            --sdk-root $FamilyRoot `
            --sdk-name $SdkName `
            --no-sync-published-version

        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
    }
}
