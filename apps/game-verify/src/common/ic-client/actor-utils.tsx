import { createActor as createInstantWinActor } from "declarations/instant_win";
import { setupIdentity } from "./identity";

export async function getInstantWinActor(canisterId: string) {
  const instant_win = createInstantWinActor(canisterId);
  await setupIdentity(instant_win);

  return instant_win;
}