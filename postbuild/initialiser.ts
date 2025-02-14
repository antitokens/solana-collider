import * as fs from "node:fs/promises";
import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { ColliderBeta } from "../target/types/collider_beta.ts";

async function loadJson<T>(path: string): Promise<T> {
  const data = await fs.readFile(path, "utf8");
  return JSON.parse(data) as T;
}

async function main() {
  // Load JSON files manually
  const keypairFile = await loadJson<number[]>("./.config/id.json");
  const idl = await loadJson<Idl>("./target/idl/collider_beta.json");

  // Setup connection and wallet
  const secretKey = new Uint8Array(keypairFile);
  const wallet = new anchor.Wallet(Keypair.fromSecretKey(secretKey));
  const connection = new Connection("https://api.devnet.solana.com");
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });

  // Load the program
  const programId = new PublicKey(
    "AMXPSQ9nWyHUqq7dB1KaPf3Wm9SMTofi7jFFGYp6pfFW"
  );
  const program = new Program(
    idl,
    programId,
    provider
  ) as unknown as Program<ColliderBeta>;

  try {
    // Find state PDA
    const [statePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );

    // Initialise program
    const tx = await program.methods
      .initialiser()
      .accounts({
        state: statePda,
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Program initialised successfully!");
    console.log("✅ Transaction signature:", tx);
  } catch (error) {
    console.error("❌ Initialisation failed:", error);
  }
}

main().catch(console.error);
