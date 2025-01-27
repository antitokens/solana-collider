import * as anchor from "@coral-xyz/anchor";
import { Program, Idl, BN } from "@coral-xyz/anchor";
import { ColliderBeta } from "../target/types/collider_beta";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  createMint, 
  createAccount, 
  mintTo, 
  getAccount
} from "@solana/spl-token";
import { expect } from "chai";

describe("collider-beta", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ColliderBeta as Program<ColliderBeta & Idl>;
  
  // Test accounts and variables
  let stateAccount: anchor.web3.Keypair;
  let pollAccount: PublicKey;
  let antiMint: PublicKey;
  let proMint: PublicKey;
  let userAntiAccount: PublicKey;
  let userProAccount: PublicKey;
  let pollAntiAccount: PublicKey;
  let pollProAccount: PublicKey;

  // Constants
  const POLL_SEED = "poll";
  const MIN_DEPOSIT = new BN(1000);
  const BASIS_POINTS = 10000;

  before(async () => {
    // Create state account
    stateAccount = anchor.web3.Keypair.generate();

    // Create token mints
    antiMint = await createMint(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      provider.wallet.publicKey,
      null,
      9,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );

    proMint = await createMint(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      provider.wallet.publicKey,
      null,
      9,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );

    // Create user token accounts
    userAntiAccount = await createAccount(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      antiMint,
      provider.wallet.publicKey
    );

    userProAccount = await createAccount(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      proMint,
      provider.wallet.publicKey
    );

    // Mint some tokens to user
    await mintTo(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      antiMint,
      userAntiAccount,
      provider.wallet.publicKey,
      1000000000
    );

    await mintTo(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      proMint,
      userProAccount,
      provider.wallet.publicKey,
      1000000000
    );
  });

  describe("Initialisation", () => {
    it("Initialises the program state", async () => {
      // Initialise program state
      await program.methods
        .initialise()
        .accounts({
          state: stateAccount.publicKey,
          authority: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([stateAccount])
        .rpc();

      // Verify state
      const state = await program.account.stateAccount.fetch(stateAccount.publicKey);
      expect(state.pollCount.toNumber()).to.equal(0);
      expect(state.authority.toString()).to.equal(provider.wallet.publicKey.toString());
    });
  });

  describe("Poll Creation", () => {
    it("Creates a new poll", async () => {
      // Get current timestamp
      const now = Math.floor(Date.now() / 1000);
      const startTime = new Date(now + 3600).toISOString(); // Start in 1 hour
      const endTime = new Date(now + 7200).toISOString();   // End in 2 hours

      // Find PDA for poll account
      const [pollPDA] = await PublicKey.findProgramAddress(
        [Buffer.from(POLL_SEED), new BN(0).toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      pollAccount = pollPDA;

      // Create payment account
      const paymentAccount = anchor.web3.Keypair.generate();
      const transferIx = SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: paymentAccount.publicKey,
        lamports: LAMPORTS_PER_SOL / 10, // 0.1 SOL
      });

      // Create poll
      await program.methods
        .createPoll(
          "Test Poll",
          "Test Description",
          startTime,
          endTime,
          null
        )
        .accounts({
          state: stateAccount.publicKey,
          poll: pollPDA,
          authority: provider.wallet.publicKey,
          payment: paymentAccount.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .preInstructions([transferIx])
        .signers([paymentAccount])
        .rpc();

      // Verify poll creation
      const poll = await program.account.pollAccount.fetch(pollPDA);
      expect(poll.index.toNumber()).to.equal(0);
      expect(poll.title).to.equal("Test Poll");
      expect(poll.description).to.equal("Test Description");
      expect(poll.startTime).to.equal(startTime);
      expect(poll.endTime).to.equal(endTime);
      expect(poll.totalAnti.toNumber()).to.equal(0);
      expect(poll.totalPro.toNumber()).to.equal(0);
      expect(poll.deposits).to.be.empty;
      expect(poll.equalised).to.be.false;
    });

    it("Fails to create poll with invalid timestamps", async () => {
      const now = Math.floor(Date.now() / 1000);
      const invalidStart = new Date(now - 3600).toISOString(); // Start 1 hour ago
      const invalidEnd = new Date(now - 1800).toISOString();   // End 30 mins ago

      try {
        await program.methods
          .createPoll(
            "Invalid Poll",
            "Invalid Description",
            invalidStart,
            invalidEnd,
            null
          )
          .accounts({
            state: stateAccount.publicKey,
            poll: anchor.web3.Keypair.generate().publicKey,
            authority: provider.wallet.publicKey,
            payment: anchor.web3.Keypair.generate().publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have failed with invalid timestamps");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Token Deposits", () => {
    before(async () => {
      // Create poll token accounts
      pollAntiAccount = await createAccount(
        provider.connection,
        (provider.wallet as anchor.Wallet).payer,
        antiMint,
        pollAccount
      );

      pollProAccount = await createAccount(
        provider.connection,
        (provider.wallet as anchor.Wallet).payer,
        proMint,
        pollAccount
      );
    });

    it("Deposits tokens successfully", async () => {
      const antiAmount = new BN(5000);
      const proAmount = new BN(3000);

      await program.methods
        .depositTokens(new BN(0), antiAmount, proAmount)
        .accounts({
          poll: pollAccount,
          authority: provider.wallet.publicKey,
          userAntiToken: userAntiAccount,
          userProToken: userProAccount,
          pollAntiToken: pollAntiAccount,
          pollProToken: pollProAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      // Verify deposits
      const poll = await program.account.pollAccount.fetch(pollAccount);
      expect(poll.totalAnti.toNumber()).to.equal(antiAmount.toNumber());
      expect(poll.totalPro.toNumber()).to.equal(proAmount.toNumber());
      expect(poll.deposits).to.have.lengthOf(1);
      
      const deposit = poll.deposits[0];
      expect(deposit.user.toString()).to.equal(provider.wallet.publicKey.toString());
      expect(deposit.antiAmount.toNumber()).to.equal(antiAmount.toNumber());
      expect(deposit.proAmount.toNumber()).to.equal(proAmount.toNumber());
      expect(deposit.withdrawn).to.be.false;
    });

    it("Fails deposit with insufficient amount", async () => {
      const smallAmount = new BN(100); // Below MIN_DEPOSIT

      try {
        await program.methods
          .depositTokens(new BN(0), smallAmount, smallAmount)
          .accounts({
            poll: pollAccount,
            authority: provider.wallet.publicKey,
            userAntiToken: userAntiAccount,
            userProToken: userProAccount,
            pollAntiToken: pollAntiAccount,
            pollProToken: pollProAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have failed with insufficient deposit");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Poll Equalisation", () => {
    it("Equalises poll with valid truth values", async () => {
      // Wait for poll to end
      await new Promise(resolve => setTimeout(resolve, 7500)); // Wait 7.5 seconds

      const truthValues = [6000, 4000]; // 60-40 split

      await program.methods
        .equaliseTokens(new BN(0), truthValues)
        .accounts({
          poll: pollAccount,
          authority: provider.wallet.publicKey,
          userAntiToken: userAntiAccount,
          userProToken: userProAccount,
          pollAntiToken: pollAntiAccount,
          pollProToken: pollProAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      // Verify equalisation
      const poll = await program.account.pollAccount.fetch(pollAccount);
      expect(poll.equalised).to.be.true;
      expect(poll.equalisationResults).to.exist;
      expect(poll.equalisationResults.truthValues).to.deep.equal(truthValues);
    });

    it("Fails equalisation with invalid truth values", async () => {
      const invalidTruthValues = [11000, 4000]; // > 10000 basis points

      try {
        await program.methods
          .equaliseTokens(new BN(0), invalidTruthValues)
          .accounts({
            poll: pollAccount,
            authority: provider.wallet.publicKey,
            userAntiToken: userAntiAccount,
            userProToken: userProAccount,
            pollAntiToken: pollAntiAccount,
            pollProToken: pollProAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have failed with invalid truth values");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Token Withdrawals", () => {
    it("Withdraws tokens after equalisation", async () => {
      const beforeAntiBalance = await getAccount(provider.connection, userAntiAccount);
      const beforeProBalance = await getAccount(provider.connection, userProAccount);

      await program.methods
        .withdrawTokens(new BN(0))
        .accounts({
          poll: pollAccount,
          authority: provider.wallet.publicKey,
          userAntiToken: userAntiAccount,
          userProToken: userProAccount,
          pollAntiToken: pollAntiAccount,
          pollProToken: pollProAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      // Verify withdrawal
      const poll = await program.account.pollAccount.fetch(pollAccount);
      expect(poll.deposits[0].withdrawn).to.be.true;

      const afterAntiBalance = await getAccount(provider.connection, userAntiAccount);
      const afterProBalance = await getAccount(provider.connection, userProAccount);

      expect(Number(afterAntiBalance.amount)).to.be.gt(Number(beforeAntiBalance.amount));
      expect(Number(afterProBalance.amount)).to.be.gt(Number(beforeProBalance.amount));
    });

    it("Fails withdrawal before equalisation on new poll", async () => {
      // Create new poll first
      const now = Math.floor(Date.now() / 1000);
      const startTime = new Date(now + 3600).toISOString();
      const endTime = new Date(now + 7200).toISOString();

      const [newPollPDA] = await PublicKey.findProgramAddress(
        [Buffer.from(POLL_SEED), new BN(1).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      const paymentAccount = anchor.web3.Keypair.generate();
      const transferIx = SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: paymentAccount.publicKey,
        lamports: LAMPORTS_PER_SOL / 10,
      });

      await program.methods
        .createPoll(
          "New Test Poll",
          "New Description",
          startTime,
          endTime,
          null
        )
        .accounts({
          state: stateAccount.publicKey,
          poll: newPollPDA,
          authority: provider.wallet.publicKey,
          payment: paymentAccount.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .preInstructions([transferIx])
        .signers([paymentAccount])
        .rpc();

      // Try to withdraw before equalisation
      try {
        await program.methods
          .withdrawTokens(new BN(1))
          .accounts({
            poll: newPollPDA,
            authority: provider.wallet.publicKey,
            userAntiToken: userAntiAccount,
            userProToken: userProAccount,
            pollAntiToken: pollAntiAccount,
            pollProToken: pollProAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have failed withdrawal before equalisation");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });
});