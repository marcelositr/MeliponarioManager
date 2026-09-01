export type MutationFlowResult =
  | { status: "success" }
  | { status: "mutation-failed"; error: unknown }
  | { status: "refresh-failed"; error: unknown };

export async function runMutationFlow(
  action: () => Promise<unknown>,
  refresh: () => Promise<unknown>,
): Promise<MutationFlowResult> {
  try {
    await action();
  } catch (error) {
    return { status: "mutation-failed", error };
  }

  try {
    await refresh();
    return { status: "success" };
  } catch (error) {
    return { status: "refresh-failed", error };
  }
}
