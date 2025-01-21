import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import { ColliderBeta } from "../target/types/collider_beta";

describe("Initialises the program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ColliderBeta as Program<ColliderBeta & Idl>;

  it("Program is initialised!", async () => {
    // Create a keypair for the new state account
    const stateAccount = anchor.web3.Keypair.generate();

    // Get the authority (provider's wallet)
    const authority = provider.wallet.publicKey;

    // Prepare the transaction
    const tx = await program.methods
      .initialise()
      .accounts({
        state: stateAccount.publicKey,
        authority: authority,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([stateAccount])
      .rpc();

    console.log("Transaction signature", tx);
  });
});
