export function wait(delayInMs: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve();
    }, delayInMs);
  });
}
