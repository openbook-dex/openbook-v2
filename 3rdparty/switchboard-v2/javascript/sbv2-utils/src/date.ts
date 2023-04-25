import type * as anchor from "@project-serum/anchor";

const padTime = (number_: number): string => {
  return number_.toString().padStart(2, "0");
};

/** Convert a Date object to YYYY-MM-DD */
export function toDateString(d: Date | undefined): string {
  if (d)
    return `${d.getFullYear()}-${padTime(d.getMonth() + 1)}-${padTime(
      d.getDate()
    )} L`;
  return "";
}

/** Convert an anchor.BN timestamp to YYYY-MM-DD */
export function anchorBNtoDateString(ts: anchor.BN): string {
  if (!ts.toNumber()) return "N/A";
  return toDateString(new Date(ts.toNumber() * 1000));
}

/** Convert a Date object to YYYY-MM-DD HH:mm:ss */
export function toDateTimeString(d: Date | undefined): string {
  if (d)
    return `${d.getFullYear()}-${padTime(d.getMonth() + 1)}-${padTime(
      d.getDate()
    )} ${padTime(d.getHours())}:${padTime(d.getMinutes())}:${padTime(
      d.getSeconds()
    )} L`;
  return "";
}

/** Convert an anchor.BN timestamp to YYYY-MM-DD HH:mm:ss */
export function anchorBNtoDateTimeString(ts: anchor.BN): string {
  if (!ts.toNumber()) return "N/A";
  return toDateTimeString(new Date(ts.toNumber() * 1000));
}
