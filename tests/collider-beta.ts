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

describe("🧪 Collider-beta end-to-end tests", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ColliderBeta as Program<ColliderBeta & Idl>;

  // Fixed keypairs for tests
  const antiMintKeypair = Keypair.fromSecretKey(
    Uint8Array.from([
      199, 248, 4, 119, 179, 209, 7, 251, 29, 104, 140, 5, 104, 142, 70, 118,
      124, 30, 234, 100, 93, 56, 177, 105, 86, 95, 183, 187, 77, 30, 146, 248,
      75, 216, 70, 100, 69, 123, 252, 137, 35, 116, 37, 57, 70, 222, 220, 169,
      103, 132, 121, 48, 61, 34, 121, 247, 62, 62, 200, 231, 57, 4, 93, 124,
    ]),
    { skipValidation: false }
  );

  const proMintKeypair = Keypair.fromSecretKey(
    Uint8Array.from([
      154, 211, 254, 243, 5, 250, 22, 77, 89, 239, 46, 250, 57, 45, 194, 24, 18,
      196, 39, 200, 37, 184, 155, 255, 83, 172, 147, 99, 16, 55, 162, 179, 83,
      14, 159, 160, 141, 181, 31, 188, 126, 1, 187, 152, 138, 51, 199, 48, 236,
      210, 29, 243, 81, 147, 101, 154, 33, 34, 191, 159, 45, 210, 243, 128,
    ]),
    { skipValidation: false }
  );

  const antitokenMultisigKeypair = Keypair.fromSecretKey(
    Uint8Array.from([
      12, 63, 179, 210, 90, 185, 236, 243, 1, 37, 19, 188, 76, 159, 88, 72, 82,
      172, 171, 255, 220, 221, 248, 84, 222, 236, 124, 122, 17, 11, 68, 197,
      101, 195, 172, 244, 31, 202, 21, 241, 93, 231, 125, 235, 92, 231, 50, 179,
      127, 190, 107, 208, 159, 17, 151, 136, 105, 43, 164, 77, 45, 59, 132, 23,
    ]),
    { skipValidation: false }
  );

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
    manager = new Keypair();
    creator = new Keypair();
    user = new Keypair();
    attacker = new Keypair();

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

  describe("○ Transaction Sizes", () => {
    it("All instruction data sizes are below 4096 bytes", async () => {
      // Test initialiseAdmin tx size
      let latestBlockhash;
      let txBuffer;

      const initAdminTx = await program.methods
        .initialiseAdmin()
        .accounts({
          admin: adminPda,
          authority: manager.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      // Get latest blockhash
      latestBlockhash = await provider.connection.getLatestBlockhash();
      initAdminTx.recentBlockhash = latestBlockhash.blockhash;
      initAdminTx.feePayer = manager.publicKey;

      txBuffer = initAdminTx.serialize({
        requireAllSignatures: false,
        verifySignatures: false,
      });

      const initAdminSize = txBuffer.length;
      expect(initAdminSize).to.be.lessThan(4096);

      // Test initialiser tx size
      const initStateTx = await program.methods
        .initialiser()
        .accounts({
          state: statePda,
          authority: manager.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      // Get latest blockhash
      latestBlockhash = await provider.connection.getLatestBlockhash();
      initStateTx.recentBlockhash = latestBlockhash.blockhash;
      initStateTx.feePayer = manager.publicKey;

      txBuffer = initStateTx.serialize({
        requireAllSignatures: false,
        verifySignatures: false,
      });

      const initStateSize = txBuffer.length;
      expect(initStateSize).to.be.lessThan(4096);

      // Test createPrediction tx size
      const createPredictionTx = await program.methods
        .createPrediction(
          "Test Prediction",
          "Test Description",
          "2025-02-01T00:00:00Z",
          "2025-03-01T00:00:00Z",
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
        .transaction();

      // Get latest blockhash
      latestBlockhash = await provider.connection.getLatestBlockhash();
      createPredictionTx.recentBlockhash = latestBlockhash.blockhash;
      createPredictionTx.feePayer = manager.publicKey;

      txBuffer = createPredictionTx.serialize({
        requireAllSignatures: false,
        verifySignatures: false,
      });

      const createPredictionSize = txBuffer.length;
      expect(createPredictionSize).to.be.lessThan(4096);

      // Test deposit tx size
      const depositTx = await program.methods
        .depositTokens(
          index,
          new BN(1000000),
          new BN(1000000),
          new BN(1739577600)
        )
        .accounts({
          prediction: predictionPda,
          authority: user.publicKey,
          userAntiToken: userAntiToken,
          userProToken: userProToken,
          predictionAntiToken: predictionAntiTokenPda,
          predictionProToken: predictionProTokenPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .transaction();

      // Get latest blockhash
      latestBlockhash = await provider.connection.getLatestBlockhash();
      depositTx.recentBlockhash = latestBlockhash.blockhash;
      depositTx.feePayer = manager.publicKey;

      txBuffer = depositTx.serialize({
        requireAllSignatures: false,
        verifySignatures: false,
      });

      const depositSize = txBuffer.length;
      expect(depositSize).to.be.lessThan(4096);

      // Test equalise tx size
      const equaliseTx = await program.methods
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
        .transaction();

      // Get latest blockhash
      latestBlockhash = await provider.connection.getLatestBlockhash();
      equaliseTx.recentBlockhash = latestBlockhash.blockhash;
      equaliseTx.feePayer = manager.publicKey;

      txBuffer = equaliseTx.serialize({
        requireAllSignatures: false,
        verifySignatures: false,
      });

      const equaliseSize = txBuffer.length;
      expect(equaliseSize).to.be.lessThan(4096);

      const remainingAccounts = [
        { pubkey: userAntiToken, isWritable: true, isSigner: false },
        { pubkey: userProToken, isWritable: true, isSigner: false },
      ];

      const bulkWithdrawTx = await program.methods
        .bulkWithdrawTokens(index)
        .accounts({
          prediction: predictionPda,
          authority: antitokenMultisigKeypair.publicKey,
          predictionAntiToken: predictionAntiTokenPda,
          predictionProToken: predictionProTokenPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .remainingAccounts(remainingAccounts)
        .transaction();

      // Get latest blockhash
      latestBlockhash = await provider.connection.getLatestBlockhash();
      bulkWithdrawTx.recentBlockhash = latestBlockhash.blockhash;
      bulkWithdrawTx.feePayer = manager.publicKey;

      txBuffer = bulkWithdrawTx.serialize({
        requireAllSignatures: false,
        verifySignatures: false,
      });

      const bulkWithdrawSize = txBuffer.length;
      expect(bulkWithdrawSize).to.be.lessThan(4096);

      // Test user withdraw tx size
      const userWithdrawTx = await program.methods
        .userWithdrawTokens(index)
        .accounts({
          state: statePda,
          prediction: predictionPda,
          authority: manager.publicKey,
          userAntiToken: userAntiToken,
          userProToken: userProToken,
          predictionAntiToken: predictionAntiTokenPda,
          predictionProToken: predictionProTokenPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          vault: antitokenMultisigKeypair.publicKey,
        })
        .transaction();

      // Get latest blockhash
      latestBlockhash = await provider.connection.getLatestBlockhash();
      userWithdrawTx.recentBlockhash = latestBlockhash.blockhash;
      userWithdrawTx.feePayer = manager.publicKey;

      txBuffer = userWithdrawTx.serialize({
        requireAllSignatures: false,
        verifySignatures: false,
      });

      const userWithdrawSize = txBuffer.length;
      expect(userWithdrawSize).to.be.lessThan(4096);

      console.log("\tInitialise Admin:", initAdminSize, "bytes");
      console.log("\tInitialiser:", initStateSize, "bytes");
      console.log("\tCreate Prediction:", createPredictionSize, "bytes");
      console.log("\tDeposit:", depositSize, "bytes");
      console.log("\tEqualise:", equaliseSize, "bytes");
      console.log("\tBulk Withdraw:", bulkWithdrawSize, "bytes");
      console.log("\tUser Withdraw:", userWithdrawSize, "bytes");
    });
  });

  describe("○ Admin", () => {
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

  describe("○ Initialisation", () => {
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

  describe("○ Prediction Creation", () => {
    it("Creates a new prediction", async () => {
      const now = Math.floor(Date.now() / 1000);
      const startTime = "2025-02-01T00:00:00Z";
      const endTime = "2025-03-01T00:00:00Z";

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

  describe("○ Token Deposits", () => {
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

  describe("○ Prediction Equalisation", () => {
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

  describe("○ Token Withdrawals", () => {
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
