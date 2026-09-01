export type LatestRequestController = {
  next: () => number;
  isCurrent: (sequence: number) => boolean;
  invalidate: () => void;
};

type LatestRequestHandlers<T> = {
  onSuccess: (value: T) => void;
  onError?: (error: unknown) => void;
  onSettled?: () => void;
};

export function createLatestRequestController(): LatestRequestController {
  let current = 0;
  return {
    next: () => ++current,
    isCurrent: (sequence) => sequence === current,
    invalidate: () => { current += 1; },
  };
}

export async function runLatestRequest<T>(
  controller: LatestRequestController,
  load: () => Promise<T>,
  handlers: LatestRequestHandlers<T>,
): Promise<"success" | "stale" | "error"> {
  const sequence = controller.next();
  try {
    const value = await load();
    if (!controller.isCurrent(sequence)) return "stale";
    handlers.onSuccess(value);
    return "success";
  } catch (error) {
    if (!controller.isCurrent(sequence)) return "stale";
    handlers.onError?.(error);
    return "error";
  } finally {
    if (controller.isCurrent(sequence)) handlers.onSettled?.();
  }
}
