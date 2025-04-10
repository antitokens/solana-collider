import * as fs from "node:fs/promises";
import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { ColliderBeta } from "../../target/types/collider_beta.ts";

async function loadJson<T>(path: string): Promise<T> {
  const data = await fs.readFile(path, "utf8");
  return JSON.parse(data) as T;
}

async function main() {
  // Declare program ID
  const programId = new PublicKey(process.env.PROGRAM_ID || "");

  // Load JSON files manually
  const keypairFile = await loadJson<number[]>("./.config/id.json");
  const idl = await loadJson<Idl>("./target/idl/collider_beta.json");

  // Setup connection and wallet
  const secretKey = new Uint8Array(keypairFile);
  const wallet = new anchor.Wallet(Keypair.fromSecretKey(secretKey));
  const connection = new Connection("http://localhost:8899");
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });

  // Load the program
  const program = new Program(
    idl,
    programId,
    provider
  ) as unknown as Program<ColliderBeta>;

  try {
    // Find admin PDA
    const [adminPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin")],
      program.programId
    );

    // Verify initialisation
    const admin = await program.account.adminAccount.fetch(adminPda);
    console.log("✅ Admin account:", {
      initialised: admin.initialised,
    });
  } catch (error) {
    console.error("❌ Verification failed:", error);
  }
}

main().catch(console.error);
