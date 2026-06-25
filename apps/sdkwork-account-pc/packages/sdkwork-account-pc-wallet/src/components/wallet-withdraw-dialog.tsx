import {
  useEffect,
  useMemo,
  useState,
} from "react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  StatusNotice,
} from "@sdkwork/ui-pc-react";
import { createSdkworkWalletPanelStyle } from "../wallet-appearance";
import type { SdkworkWalletController } from "../wallet-controller";
import { useSdkworkWalletControllerState } from "../wallet-controller";
import { createDefaultSdkworkWalletWithdrawDestinations } from "../wallet";
import { useSdkworkWalletIntl } from "../wallet-intl";

export interface SdkworkWalletWithdrawDialogProps {
  controller: SdkworkWalletController;
  onOpenChange?: (open: boolean) => void;
  open: boolean;
}

function sanitizeAmount(value: string): string {
  const normalized = value.replaceAll(/[^\d.]/g, "");
  const [integerPart, ...fractionParts] = normalized.split(".");
  const fractionPart = fractionParts.join("").slice(0, 2);
  return fractionPart ? `${integerPart}.${fractionPart}` : integerPart;
}

export function SdkworkWalletWithdrawDialog({
  controller,
  onOpenChange,
  open,
}: SdkworkWalletWithdrawDialogProps) {
  const state = useSdkworkWalletControllerState(controller);
  const destinations = useMemo(() => createDefaultSdkworkWalletWithdrawDestinations(), []);
  const [amountInput, setAmountInput] = useState("");
  const [accountName, setAccountName] = useState("");
  const [accountNo, setAccountNo] = useState("");
  const [bankName, setBankName] = useState("");
  const [requestNo, setRequestNo] = useState("");
  const [selectedDestinationCode, setSelectedDestinationCode] = useState<string>(destinations[0]?.code ?? "");
  const {
    copy,
    formatCurrencyCny,
    formatProjectedBalance,
    formatWithdrawDestinationDescription,
    formatWithdrawDestinationLabel,
    formatWithdrawRemarks,
  } = useSdkworkWalletIntl();
  const amountCny = Number.parseFloat(amountInput || "0");
  const trimmedAccountName = accountName.trim();
  const trimmedAccountNo = accountNo.trim();
  const trimmedBankName = bankName.trim();
  const trimmedRequestNo = requestNo.trim();
  const requestNoValid = !trimmedRequestNo || /^[A-Za-z0-9_-]{6,64}$/.test(trimmedRequestNo);
  const requiresBankName = selectedDestinationCode === "bank_account";
  const projectedBalance = Number.isFinite(amountCny)
    ? Math.max(0, Number((state.overview.account.cashAvailable - amountCny).toFixed(2)))
    : state.overview.account.cashAvailable;

  useEffect(() => {
    if (!open) {
      return;
    }

    setAmountInput("");
    setAccountName("");
    setAccountNo("");
    setBankName("");
    setRequestNo("");
    setSelectedDestinationCode(destinations[0]?.code ?? "");
  }, [destinations, open]);

  const selectedDestination = destinations.find((destination) => destination.code === selectedDestinationCode) ?? null;
  const canSubmit = useMemo(
    () =>
      state.overview.isAuthenticated
      && Number.isFinite(amountCny)
      && amountCny > 0
      && amountCny <= state.overview.account.cashAvailable
      && Boolean(trimmedAccountName)
      && Boolean(trimmedAccountNo)
      && (!requiresBankName || Boolean(trimmedBankName))
      && Boolean(selectedDestinationCode)
      && requestNoValid
      && !state.isMutating,
    [
      amountCny,
      requestNoValid,
      requiresBankName,
      selectedDestinationCode,
      state.isMutating,
      state.overview.account.cashAvailable,
      state.overview.isAuthenticated,
      trimmedAccountName,
      trimmedAccountNo,
      trimmedBankName,
    ],
  );

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent className="w-[min(92vw,54rem)]">
        <DialogHeader>
          <DialogTitle>{copy.withdrawDialog.title}</DialogTitle>
          <DialogDescription>
            {copy.withdrawDialog.description}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-5">
          <div
            className="rounded-[1.25rem] border p-4"
            style={createSdkworkWalletPanelStyle("brand", {
              backgroundWeight: 8,
              borderWeight: 24,
              surfaceWeight: 92,
            })}
          >
            <div className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
              {copy.withdrawDialog.availableCashEyebrow}
            </div>
            <div className="mt-3 text-3xl font-semibold text-[var(--sdk-color-text-primary)]">
              {formatCurrencyCny(state.overview.account.cashAvailable)}
            </div>
            <div className="mt-2 text-sm text-[var(--sdk-color-text-secondary)]">
              {formatProjectedBalance(projectedBalance)}
            </div>
          </div>

          {!state.overview.isAuthenticated ? (
            <StatusNotice title={copy.withdrawDialog.signInRequiredTitle} tone="warning">
              {copy.withdrawDialog.signInRequiredDescription}
            </StatusNotice>
          ) : null}

          {Number.isFinite(amountCny) && amountCny > state.overview.account.cashAvailable ? (
            <StatusNotice title={copy.withdrawDialog.insufficientTitle} tone="warning">
              {copy.withdrawDialog.insufficientDescription}
            </StatusNotice>
          ) : null}

          {!requestNoValid ? (
            <StatusNotice title={copy.withdrawDialog.invalidRequestNoTitle} tone="warning">
              {copy.withdrawDialog.invalidRequestNoDescription}
            </StatusNotice>
          ) : null}

          <div className="space-y-2">
            <label className="text-sm font-medium text-[var(--sdk-color-text-primary)]" htmlFor="sdkwork-wallet-withdraw-amount">
              {copy.withdrawDialog.amountLabel}
            </label>
            <Input
              id="sdkwork-wallet-withdraw-amount"
              inputMode="decimal"
              onChange={(event) => setAmountInput(sanitizeAmount(event.target.value))}
              placeholder={copy.withdrawDialog.amountPlaceholder}
              type="text"
              value={amountInput}
            />
          </div>

          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <label className="text-sm font-medium text-[var(--sdk-color-text-primary)]" htmlFor="sdkwork-wallet-withdraw-account-name">
                {copy.withdrawDialog.accountNameLabel}
              </label>
              <Input
                id="sdkwork-wallet-withdraw-account-name"
                onChange={(event) => setAccountName(event.target.value)}
                placeholder={copy.withdrawDialog.accountNamePlaceholder}
                type="text"
                value={accountName}
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-[var(--sdk-color-text-primary)]" htmlFor="sdkwork-wallet-withdraw-account-number">
                {copy.withdrawDialog.accountNoLabel}
              </label>
              <Input
                id="sdkwork-wallet-withdraw-account-number"
                onChange={(event) => setAccountNo(event.target.value)}
                placeholder={copy.withdrawDialog.accountNoPlaceholder}
                type="text"
                value={accountNo}
              />
            </div>
          </div>

          <div>
            <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
              {copy.withdrawDialog.withdrawDestinationLabel}
            </div>
            <div className="mt-3 grid gap-3 sm:grid-cols-3">
              {destinations.map((destination) => (
                <Button
                  className="h-auto min-h-24 flex-col items-start justify-start gap-2 px-4 py-4 text-left"
                  key={destination.id}
                  onClick={() => setSelectedDestinationCode(destination.code)}
                  type="button"
                  variant={selectedDestinationCode === destination.code ? "secondary" : "outline"}
                >
                  <span className="text-sm font-semibold">{formatWithdrawDestinationLabel(destination.code)}</span>
                  <span className="whitespace-normal text-xs font-normal text-[var(--sdk-color-text-secondary)]">
                    {formatWithdrawDestinationDescription(destination.code)}
                  </span>
                </Button>
              ))}
            </div>
          </div>

          <div className="grid gap-4 sm:grid-cols-2">
            {requiresBankName ? (
              <div className="space-y-2">
                <label className="text-sm font-medium text-[var(--sdk-color-text-primary)]" htmlFor="sdkwork-wallet-withdraw-bank-name">
                  {copy.withdrawDialog.bankNameLabel}
                </label>
                <Input
                  id="sdkwork-wallet-withdraw-bank-name"
                  onChange={(event) => setBankName(event.target.value)}
                  placeholder={copy.withdrawDialog.bankNamePlaceholder}
                  type="text"
                  value={bankName}
                />
              </div>
            ) : null}

            <div className="space-y-2">
              <label className="text-sm font-medium text-[var(--sdk-color-text-primary)]" htmlFor="sdkwork-wallet-withdraw-request-no">
                {copy.withdrawDialog.requestNoLabel}
              </label>
              <Input
                id="sdkwork-wallet-withdraw-request-no"
                onChange={(event) => setRequestNo(event.target.value)}
                placeholder={copy.withdrawDialog.requestNoPlaceholder}
                type="text"
                value={requestNo}
              />
            </div>
          </div>

          {selectedDestination ? (
            <div className="rounded-[1rem] border border-[var(--sdk-color-border-subtle)] bg-[var(--sdk-color-surface-panel)] px-4 py-3 text-sm text-[var(--sdk-color-text-secondary)]">
              {copy.withdrawDialog.payoutRailLabel}:{" "}
              <span className="font-semibold text-[var(--sdk-color-text-primary)]">
                {formatWithdrawDestinationLabel(selectedDestination.code)}
              </span>
            </div>
          ) : null}
        </div>

        <DialogFooter>
          <Button onClick={() => onOpenChange?.(false)} type="button" variant="ghost">
            {copy.actions.cancel}
          </Button>
          <Button
            disabled={!canSubmit}
            loading={state.isMutating}
            onClick={() => {
              if (!canSubmit) {
                return;
              }

              void controller.withdrawCash({
                accountName: trimmedAccountName,
                accountNo: trimmedAccountNo,
                amountCny,
                ...(requiresBankName ? { bankName: trimmedBankName } : {}),
                destinationCode: selectedDestinationCode,
                remarks: selectedDestination
                  ? formatWithdrawRemarks(formatWithdrawDestinationLabel(selectedDestination.code))
                  : undefined,
                requestNo: trimmedRequestNo || undefined,
              });
            }}
            type="button"
          >
            {copy.actions.confirmWithdraw}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
