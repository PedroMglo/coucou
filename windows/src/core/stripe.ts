/** The native poller formats the minor units so both views show the same amount. */
export function stripePaymentAmount(payment: Record<string, unknown>): string {
  const { amountText, currency } = payment;
  if (typeof amountText !== "string" || typeof currency !== "string") return "—";
  return `${amountText.startsWith("-") ? "" : "+"}${amountText} ${currency.toUpperCase()}`;
}
