import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { AccountAppShell } from "@sdkwork/account-pc-shell";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <AccountAppShell />
  </StrictMode>,
);
