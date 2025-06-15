import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { JupiterDelegate } from "../target/types/jupiter_delegate";

describe("jupiter-delegate", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.jupiterDelegate as Program<JupiterDelegate>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initConfig(new anchor.BN(60)) // Set cooldown to 60 seconds
      .rpc();
    console.log("Your transaction signature", tx);
  });
});
