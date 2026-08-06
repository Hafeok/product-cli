// Fixture: unusedParam trips no-unused-params (enabled in bicepconfig.json
// beside it); no-hardcoded-env-urls is turned off there — a config
// suppression that must cite a risk record.
// Regenerate the committed SARIF (verified invocation, see README §ddd):
//   bicep lint main.bicep --diagnostics-format sarif > bicep.sarif

param unusedParam string = 'never-read'
param location string = 'westeurope'

resource stg 'Microsoft.Storage/storageAccounts@2023-01-01' = {
  name: 'dddfixture001'
  location: location
  sku: {
    name: 'Standard_LRS'
  }
  kind: 'StorageV2'
}

output storageId string = stg.id
