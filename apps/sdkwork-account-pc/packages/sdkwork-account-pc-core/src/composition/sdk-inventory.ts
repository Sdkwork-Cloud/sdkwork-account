export function listSdkworkCoreSdkInventory() {
  return [
    {
      packageName: "@sdkwork/account-app-sdk",
      authority: "sdkwork-account.app",
      surface: "app-api",
    },
    {
      packageName: "@sdkwork/account-backend-sdk",
      authority: "sdkwork-account.backend",
      surface: "backend-api",
    },
  ] as const;
}
