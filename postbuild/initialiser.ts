import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { ColliderBeta } from "../target/types/collider_beta";

// Configure your RPC endpoint
const ENDPOINT = "https://api.devnet.solana.com"; // Change this to your desired endpoint

async function main() {
  // Setup connection and wallet
  const connection = new Connection(ENDPOINT);
  const keypairFile = require("../.config/id.json");
  const wallet = new anchor.Wallet(
    Keypair.fromSecretKey(new Uint8Array(keypairFile))
  );

  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });

  // Load the program
  const programId = new PublicKey(
    "AMXPSQ9nWyHUqq7dB1KaPf3Wm9SMTofi7jFFGYp6pfFW"
  );
  const program = new Program(
    require("../target/idl/collider_beta.json"),
    programId
  ) as Program<ColliderBeta>;

  try {
    // Find state PDA
    const [statePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );

    // Initialize program
    const tx = await program.methods
      .initialiser()
      .accounts({
        state: statePda,
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Program initialised successfully!");
    console.log("Transaction signature:", tx);

    // Verify initialization
    const state = await program.account.stateAccount.fetch(statePda);
    console.log("State account:", {
      pollIndex: state.pollIndex.toString(),
      authority: state.authority.toString(),
    });
  } catch (error) {
    console.error("Initialisation failed:", error);
  }
}

main().catch(console.error);
