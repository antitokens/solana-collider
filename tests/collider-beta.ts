import * as anchor from "@coral-xyz/anchor";
import { Program, Idl, BN } from "@coral-xyz/anchor";
import { ColliderBeta } from "../target/types/collider_beta.ts";
import {
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
  Keypair,
  Signer,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { expect } from "chai";
import fs from "fs/promises";

async function loadJson<T>(path: string): Promise<T> {
  const data = await fs.readFile(path, "utf8");
  return JSON.parse(data) as T;
}

// Declare variables to hold keypair data and initialized keypairs
let antiMintSecretKey: number[];
let proMintSecretKey: number[];
let vaultSecretKey: number[];
let creatorSecretKey: number[];
let managerSecretKey: number[];
let userSecretKey: number[];
let deployerSecretKey: number[];

let antiMintKeypair: Keypair;
let proMintKeypair: Keypair;
let antitokenMultisigKeypair: Keypair;
let creatorKeypair: Keypair;
let managerKeypair: Keypair;
let userKeypair: Keypair;
let deployerKeypair: Keypair;

// Load keypairs before tests begin
before(async () => {
  antiMintSecretKey = await loadJson<number[]>(".config/dAnti/token.json");
  proMintSecretKey = await loadJson<number[]>(".config/dPro/token.json");
  vaultSecretKey = await loadJson<number[]>(".config/dVault/id.json");
  creatorSecretKey = await loadJson<number[]>(".config/dCreator/id.json");
  managerSecretKey = await loadJson<number[]>(".config/dManager/id.json");
  userSecretKey = await loadJson<number[]>(".config/dUser/id.json");
  deployerSecretKey = await loadJson<number[]>(".config/id.json");

  antiMintKeypair = Keypair.fromSecretKey(Uint8Array.from(antiMintSecretKey), {
    skipValidation: false,
  });

  proMintKeypair = Keypair.fromSecretKey(Uint8Array.from(proMintSecretKey), {
    skipValidation: false,
  });

  antitokenMultisigKeypair = Keypair.fromSecretKey(
    Uint8Array.from(vaultSecretKey),
    { skipValidation: false }
  );

  creatorKeypair = Keypair.fromSecretKey(Uint8Array.from(creatorSecretKey), {
    skipValidation: false,
  });

  managerKeypair = Keypair.fromSecretKey(Uint8Array.from(managerSecretKey), {
    skipValidation: false,
  });

  userKeypair = Keypair.fromSecretKey(Uint8Array.from(userSecretKey), {
    skipValidation: false,
  });

  deployerKeypair = Keypair.fromSecretKey(Uint8Array.from(deployerSecretKey), {
    skipValidation: false,
  });
});

describe("collider-beta", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ColliderBeta as Program<ColliderBeta & Idl>;

  let manager: Keypair;
  let creator: Keypair;
  let user: Keypair;
  let attacker: Keypair;

  // PDAs and accounts
  let adminPda: PublicKey;
  let statePda: PublicKey;
  let predictionPda: PublicKey;
  let predictionPda2: PublicKey;
  let predictionAntiTokenPda: PublicKey;
  let predictionProTokenPda: PublicKey;

  let userAntiToken: PublicKey;
  let userProToken: PublicKey;

  const index = new BN(0);

  before(async () => {
    // Create test keypairs
    manager = managerKeypair;
    creator = creatorKeypair;
    user = userKeypair;
    attacker = new Keypair(); // Randomly generated

    // Airdrop SOL to all accounts
    const airdropAmount = 10 * LAMPORTS_PER_SOL;
    const accounts = [
      manager,
      creator,
      user,
      attacker,
      antitokenMultisigKeypair,
    ];

    for (const account of accounts) {
      const sig = await provider.connection.requestAirdrop(
        account.publicKey,
        airdropAmount
      );
      await provider.connection.confirmTransaction(sig);
    }

    // Find PDAs
    [adminPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin")],
      program.programId
    );

    [statePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );

    [predictionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("prediction"), index.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    [predictionAntiTokenPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("anti_token"), index.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    [predictionProTokenPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pro_token"), index.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    // Initialise token mints
    await createMint(
      provider.connection,
      manager,
      manager.publicKey,
      null,
      9,
      antiMintKeypair,
      undefined,
      TOKEN_PROGRAM_ID
    );

    await createMint(
      provider.connection,
      manager,
      manager.publicKey,
      null,
      9,
      proMintKeypair,
      undefined,
      TOKEN_PROGRAM_ID
    );

    // Create user token accounts
    userAntiToken = await createAccount(
      provider.connection,
      manager,
      antiMintKeypair.publicKey,
      user.publicKey
    );

    userProToken = await createAccount(
      provider.connection,
      manager,
      proMintKeypair.publicKey,
      user.publicKey
    );

    // Mint tokens to user
    await mintTo(
      provider.connection,
      manager,
      antiMintKeypair.publicKey,
      userAntiToken,
      manager.publicKey,
      10_000_000_000
    );

    await mintTo(
      provider.connection,
      manager,
      proMintKeypair.publicKey,
      userProToken,
      manager.publicKey,
      10_000_000_000
    );
  });

  describe("Admin", () => {
    it("Initialises the admin state", async () => {
      await program.methods
        .initialiseAdmin()
        .accounts({
          admin: adminPda,
          authority: manager.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([manager])
        .rpc();

      const admin = await program.account.adminAccount.fetch(adminPda);
      expect(admin.initialised).to.be.true;
    });
  });

  describe("Initialisation", () => {
    it("Initialises the program state", async () => {
      await program.methods
        .initialiser()
        .accounts({
          state: statePda,
          authority: manager.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([manager])
        .rpc();

      const state = await program.account.stateAccount.fetch(statePda);
      expect(Number(state.index)).to.equal(0);
      expect(state.authority?.toString()).to.equal(
        manager.publicKey.toString()
      );
    });
  });

  describe("Prediction Creation", () => {
    it("Creates a new prediction", async () => {
      const now = Math.floor(Date.now() / 1000);
      const startTime = "2025-02-01T00:00:00Z";
      const endTime = "2025-03-01T00:00:00Z";

      console.log("🔍 State PDA:", statePda.toBase58());
      console.log("🔍 Prediction PDA:", predictionPda.toBase58());
      console.log(
        "🔍 Prediction $ANTI PDA:",
        predictionAntiTokenPda.toBase58()
      );
      console.log("🔍 Prediction $PRO PDA:", predictionProTokenPda.toBase58());
      console.log("🔍 $ANTI MINT:", antiMintKeypair.publicKey.toBase58());
      console.log("🔍 $PRO MINT:", proMintKeypair.publicKey.toBase58());
      console.log("🔍 VAULT:", antitokenMultisigKeypair.publicKey.toBase58());
      console.log("🔍 CREATOR:", creator.publicKey.toBase58());

      await program.methods
        .createPrediction(
          "Test Prediction",
          "Test Description",
          startTime,
          endTime,
          null,
          new BN(1736899200) // Fixed timestamp for testing
        )
        .accounts({
          state: statePda,
          prediction: predictionPda,
          authority: creator.publicKey,
          predictionAntiToken: predictionAntiTokenPda,
          predictionProToken: predictionProTokenPda,
          antiMint: antiMintKeypair.publicKey,
          proMint: proMintKeypair.publicKey,
          vault: antitokenMultisigKeypair.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([creator])
        .rpc();

      const prediction = await program.account.predictionAccount.fetch(
        predictionPda
      );
      expect(Number(prediction.index)).to.equal(0);
      expect(prediction.title).to.equal("Test Prediction");
      expect(prediction.description).to.equal("Test Description");
      expect(prediction.startTime).to.equal(startTime);
      expect(prediction.endTime).to.equal(endTime);
    });
  });

  describe("Token Deposits", () => {
    it("Deposits tokens successfully", async () => {
      const anti = new BN(7_000_000_000);
      const pro = new BN(3_000_000_000);

      await program.methods
        .depositTokens(index, anti, pro, new BN(1739577600))
        .accounts({
          prediction: predictionPda,
          authority: user.publicKey,
          userAntiToken: userAntiToken,
          userProToken: userProToken,
          predictionAntiToken: predictionAntiTokenPda,
          predictionProToken: predictionProTokenPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      const prediction = await program.account.predictionAccount.fetch(
        predictionPda
      );
      expect(Number(prediction.anti)).to.equal(anti.toNumber());
      expect(Number(prediction.pro)).to.equal(pro.toNumber());
      expect(prediction.deposits).to.have.lengthOf(1);
    });
  });

  describe("Prediction Equalisation", () => {
    it("Equalises prediction with truth", async () => {
      await program.methods
        .equaliseTokens(index, [new BN(6000), new BN(4000)], new BN(1741996800))
        .accounts({
          prediction: predictionPda,
          authority: manager.publicKey,
          userAntiToken: userAntiToken,
          userProToken: userProToken,
          predictionAntiToken: predictionAntiTokenPda,
          predictionProToken: predictionProTokenPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([manager])
        .rpc();

      const prediction = await program.account.predictionAccount.fetch(
        predictionPda
      );
      expect(prediction.equalised).to.be.true;
      expect(prediction.equalisation).to.exist;
    });
  });

  describe("Token Withdrawals", () => {
    it("Withdraws tokens after equalisation", async () => {
      const beforeAntiBalance = await getAccount(
        provider.connection,
        userAntiToken
      );

      const beforeProBalance = await getAccount(
        provider.connection,
        userProToken
      );

      const remainingAccounts = [
        { pubkey: userAntiToken, isWritable: true, isSigner: false },
        { pubkey: userProToken, isWritable: true, isSigner: false },
      ];

      await program.methods
        .bulkWithdrawTokens(index)
        .accounts({
          prediction: predictionPda,
          authority: antitokenMultisigKeypair.publicKey,
          predictionAntiToken: predictionAntiTokenPda,
          predictionProToken: predictionProTokenPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .remainingAccounts(remainingAccounts)
        .signers([antitokenMultisigKeypair])
        .rpc();

      const prediction = await program.account.predictionAccount.fetch(
        predictionPda
      );
      expect(prediction.deposits[0].withdrawn).to.be.true;

      const afterAntiBalance = await getAccount(
        provider.connection,
        userAntiToken
      );
      const afterProBalance = await getAccount(
        provider.connection,
        userProToken
      );

      expect(Number(afterAntiBalance.amount)).to.be.gt(
        Number(beforeAntiBalance.amount)
      );
      expect(Number(afterProBalance.amount)).to.be.gt(
        Number(beforeProBalance.amount)
      );
    });
  });
});
