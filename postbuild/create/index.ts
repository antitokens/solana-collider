import * as fs from "node:fs/promises";
import * as dotenv from "dotenv";
import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ColliderBeta } from "../../target/types/collider_beta.ts";

dotenv.config();

const SOLANA_API = `${process.env.SOLANA_API}`;

async function loadJson<T>(path: string): Promise<T> {
  const data = await fs.readFile(path, "utf8");
  return JSON.parse(data) as T;
}

async function main() {
  // Declare program ID
  const programId = new PublicKey(
    "3zKqVU2RiWXPe3bvTjQ869UF6qng2LoGBKEFmUqh8BzA"
  );

  /// Accounts that must be created externally and/or exist beforehand
  // Declare $ANTI mint address
  const ANTI_MINT = new PublicKey(`${process.env.ANTI_TOKEN_MINT}`);
  // Declare $PRO mint address
  const PRO_MINT = new PublicKey(`${process.env.PRO_TOKEN_MINT}`);
  // Declare Antitoken vault address
  const VAULT = new PublicKey(`${process.env.VAULT}`);

  // Load JSON files manually
  const keypairFile = await loadJson<number[]>("./.config/user.json");
  const idl = await loadJson<Idl>("./target/idl/collider_beta.json");

  try {
    // Setup connection and wallet
    const secretKey = new Uint8Array(keypairFile);
    const creator = Keypair.fromSecretKey(secretKey);
    const wallet = new anchor.Wallet(creator);
    const connection = new Connection(SOLANA_API);
    const provider = new anchor.AnchorProvider(connection, wallet, {
      commitment: "confirmed",
    });

    // Load the program
    const program = new Program(
      idl,
      programId,
      provider
    ) as unknown as Program<ColliderBeta>;

    // Find state PDA
    const [statePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );

    const state = await program.account.stateAccount.fetch(statePda);

    const [predictionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("prediction"), state.index.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const [predictionAntiTokenPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("anti_token"), state.index.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const [predictionProTokenPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pro_token"), state.index.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const prediction = {
      title: "Test Prediction",
      description: "Test Description",
      startTime: "2025-03-01T00:00:00Z",
      endTime: "2025-04-01T00:00:00Z",
      option: null,
    };

    // Initialise program
    const tx = await program.methods
      .createPrediction(
        prediction.title,
        prediction.description,
        prediction.startTime,
        prediction.endTime,
        prediction.option
      )
      .accounts({
        state: statePda,
        prediction: predictionPda,
        authority: creator.publicKey,
        predictionAntiToken: predictionAntiTokenPda,
        predictionProToken: predictionProTokenPda,
        antiMint: ANTI_MINT,
        proMint: PRO_MINT,
        vault: VAULT,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([creator])
      .rpc();

    console.log("✅ Program initialised successfully!");
    console.log("✅ Transaction signature:", tx);
  } catch (error) {
    console.error("❌ Initialisation failed:", error);
  }
}

main().catch(console.error);
