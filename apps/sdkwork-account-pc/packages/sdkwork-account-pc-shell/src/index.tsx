import { sdkworkAccountPcRuntimeIdentity } from "@sdkwork/account-pc-core";

export function AccountAppShell() {
  return (
    <main className="account-shell">
      <section className="account-card">
        <h1>SDKWork Account</h1>
        <p>{sdkworkAccountPcRuntimeIdentity.appKey}</p>
        <p>Account and wallet capability PC surface — aligned with sdkwork-specs building-block model.</p>
      </section>
    </main>
  );
}
