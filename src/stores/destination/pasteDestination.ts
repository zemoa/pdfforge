import { destinationClient } from "../../application/destinationClient";

export function createPastedDestinationIntent(
  isUnavailable: () => boolean,
  applyDestination: (path: string) => void,
) {
  return async function pasteDestination(pastedText: string) {
    if (isUnavailable()) return;

    let destination = pastedText;
    try {
      destination = (await destinationClient.resolvePastedFolder()) ?? pastedText;
    } catch {
      // A clipboard lock must not prevent an ordinary text paste.
    }

    if (!isUnavailable()) applyDestination(destination);
  };
}
