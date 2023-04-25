/** Sleep for a given number of milliseconds
 * @param ms number of milliseconds to sleep for
 * @return a promise that resolves when the sleep interval has elapsed
 */
export const sleep = (ms: number): Promise<any> =>
  new Promise((s) => setTimeout(s, ms));

/** Returns a promise that resolves successfully if returned before the given timeout has elapsed.
 * @param ms the number of milliseconds before the promise expires
 * @param promise the promise to wait for
 * @param timeoutError the error to throw if the promise expires
 * @return the promise result
 */
export async function promiseWithTimeout<T>(
  ms: number,
  promise: Promise<T>,
  timeoutError = new Error("timeoutError")
): Promise<T> {
  // create a promise that rejects in milliseconds
  const timeout = new Promise<never>((_, reject) => {
    setTimeout(() => {
      reject(timeoutError);
    }, ms);
  });

  return Promise.race<T>([promise, timeout]);
}
