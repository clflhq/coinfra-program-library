import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { NftBarter } from "../target/types/nft_barter";
import { PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
  getAccount,
  Account,
} from "@solana/spl-token";
import { assert } from "chai";

describe("anchor-escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.NftBarter as Program<NftBarter>;

  let mintA: anchor.web3.PublicKey = null;
  let mintB: anchor.web3.PublicKey = null;
  let mintC: anchor.web3.PublicKey = null;
  let mintD: anchor.web3.PublicKey = null;
  let mintE: anchor.web3.PublicKey = null;

  let initializerTokenAccountA: Account = null;
  let initializerTokenAccountB: Account = null;
  let initializerTokenAccountC: Account = null;
  let initializerTokenAccountD: Account = null;
  let initializerTokenAccountE: Account = null;

  let takerTokenAccountA: Account = null;
  let takerTokenAccountB: Account = null;
  let takerTokenAccountC: Account = null;
  let takerTokenAccountD: Account = null;
  let takerTokenAccountE: Account = null;

  let vaultAccountPdaA: anchor.web3.PublicKey = null; // initializerがmintAを預ける用のvault vaultはPDAにする必要がない
  let vaultAccountBumpA: number = null;

  let vaultAccountPdaB: anchor.web3.PublicKey = null; // initializerがmintBを預ける用のvault
  let vaultAccountBumpB: number = null;

  let vaultAccountPdaC: anchor.web3.PublicKey = null; // initializerがmintCを預ける用のvault
  let vaultAccountBumpC: number = null;

  let vaultAccountPdaD: anchor.web3.PublicKey = null; // initializerがmintDを預ける用のvault
  let vaultAccountBumpD: number = null;

  let vaultAccountPdaE: anchor.web3.PublicKey = null; // initializerがmintEを預ける用のvault
  let vaultAccountBumpE: number = null;

  let vaultAuthorityPda: anchor.web3.PublicKey = null;
  let vaultAuthorityBump: number = null;

  let initializerNftAmount = 2;
  let takerNftAmount = 3;

  const initializerStartSolAmount = 2_000_000_000; // lamport
  const takerStartSolAmount = 5_000_000_000; // lamport
  const initializerAdditionalSolAmount = 500_000_000; // lamport
  const takerAdditionalSolAmount = 1_000_000_000; // lamport

  const escrowAccount: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  const payer: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  const mintAuthority: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  const initializerMainAccount: anchor.web3.Keypair =
    anchor.web3.Keypair.generate();
  const takerMainAccount: anchor.web3.Keypair = anchor.web3.Keypair.generate();

  it("Initialize program state", async () => {
    console.log("start airdrop");

    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10_000_000_000),
      "confirmed"
    );

    // Fund Main Accounts
    await provider.send(
      (() => {
        const tx = new Transaction();
        tx.add(
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: initializerMainAccount.publicKey,
            lamports: initializerStartSolAmount,
          }),
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: takerMainAccount.publicKey,
            lamports: takerStartSolAmount,
          })
        );
        return tx;
      })(),
      [payer]
    );

    console.log("start createMint");

    mintA = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    mintB = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    mintC = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    mintD = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    mintE = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    console.log("start getOrCreateAssociatedTokenAccount");

    // mintA mintBはinitializerが保有している
    initializerTokenAccountA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      initializerMainAccount,
      mintA,
      initializerMainAccount.publicKey
    );
    initializerTokenAccountB = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      initializerMainAccount,
      mintB,
      initializerMainAccount.publicKey
    );
    initializerTokenAccountC = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      initializerMainAccount,
      mintC,
      initializerMainAccount.publicKey
    );
    initializerTokenAccountD = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      initializerMainAccount,
      mintD,
      initializerMainAccount.publicKey
    );
    initializerTokenAccountE = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      initializerMainAccount,
      mintE,
      initializerMainAccount.publicKey
    );

    // mintC mintD mintEはtakerが保有している
    takerTokenAccountA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      takerMainAccount,
      mintA,
      takerMainAccount.publicKey
    );
    takerTokenAccountB = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      takerMainAccount,
      mintB,
      takerMainAccount.publicKey
    );
    takerTokenAccountC = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      takerMainAccount,
      mintC,
      takerMainAccount.publicKey
    );
    takerTokenAccountD = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      takerMainAccount,
      mintD,
      takerMainAccount.publicKey
    );
    takerTokenAccountE = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      takerMainAccount,
      mintE,
      takerMainAccount.publicKey
    );

    console.log("start mintTo");

    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintA, // Mint for the account
      initializerTokenAccountA.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );

    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintB, // Mint for the account
      initializerTokenAccountB.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );

    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintC, // Mint for the account
      takerTokenAccountC.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );

    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintD, // Mint for the account
      takerTokenAccountD.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );

    await mintTo(
      provider.connection,
      payer, // Payer of the transaction fees
      mintE, // Mint for the account
      takerTokenAccountE.address, // Address of the account to mint to
      mintAuthority, // Minting authority
      1 // Amount to mint
    );

    console.log("start assertion");

    // check token accounts
    let _initializerTokenAccountA = await getAccount(
      provider.connection,
      initializerTokenAccountA.address
    );
    assert.ok(Number(_initializerTokenAccountA.amount) === 1);

    let _initializerTokenAccountB = await getAccount(
      provider.connection,
      initializerTokenAccountB.address
    );
    assert.ok(Number(_initializerTokenAccountB.amount) === 1);

    let _initializerTokenAccountC = await getAccount(
      provider.connection,
      initializerTokenAccountC.address
    );
    assert.ok(Number(_initializerTokenAccountC.amount) === 0);

    let _initializerTokenAccountD = await getAccount(
      provider.connection,
      initializerTokenAccountD.address
    );
    assert.ok(Number(_initializerTokenAccountD.amount) === 0);

    let _initializerTokenAccountE = await getAccount(
      provider.connection,
      initializerTokenAccountE.address
    );
    assert.ok(Number(_initializerTokenAccountE.amount) === 0);

    let _takerTokenAccountA = await getAccount(
      provider.connection,
      takerTokenAccountA.address
    );
    assert.ok(Number(_takerTokenAccountA.amount) === 0);

    let _takerTokenAccountB = await getAccount(
      provider.connection,
      takerTokenAccountB.address
    );
    assert.ok(Number(_takerTokenAccountB.amount) === 0);

    let _takerTokenAccountC = await getAccount(
      provider.connection,
      takerTokenAccountC.address
    );
    assert.ok(Number(_takerTokenAccountC.amount) === 1);

    let _takerTokenAccountD = await getAccount(
      provider.connection,
      takerTokenAccountD.address
    );
    assert.ok(Number(_takerTokenAccountD.amount) === 1);

    let _takerTokenAccountE = await getAccount(
      provider.connection,
      takerTokenAccountE.address
    );
    assert.ok(Number(_takerTokenAccountE.amount) === 1);
  });

  it("Initialize escrow", async () => {
    console.log("start creating PDAs");

    const [_vaultAccountPdaA, _vaultAccountBumpA] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountA.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaA = _vaultAccountPdaA;
    vaultAccountBumpA = _vaultAccountBumpA;
    console.log("initializerTokenAccountA", initializerTokenAccountA);
    console.log("vaultAccountPdaA", vaultAccountPdaA);
    console.log("vaultAccountBumpA", vaultAccountBumpA);

    const [_vaultAccountPdaB, _vaultAccountBumpB] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountB.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaB = _vaultAccountPdaB;
    vaultAccountBumpB = _vaultAccountBumpB;
    console.log("initializerTokenAccountB", initializerTokenAccountB);
    console.log("vaultAccountPdaB", vaultAccountPdaB);
    console.log("vaultAccountBumpB", vaultAccountBumpB);

    const vaultAccountBumps: number[] = [vaultAccountBumpA, vaultAccountBumpB];
    console.log("vaultAccountBumps", vaultAccountBumps);

    const [_vaultAuthorityPda, _vaultAuthorityBump] =
      await PublicKey.findProgramAddress(
        [
          anchor.utils.bytes.utf8.encode("vault-authority"),
          initializerMainAccount.publicKey.toBuffer(),
          takerMainAccount.publicKey.toBuffer(),
        ], // sampleコードではBufferが書いてあったがBufferはいらないと思われる anchor bookにはない
        program.programId
      );
    vaultAuthorityPda = _vaultAuthorityPda;
    vaultAuthorityBump = _vaultAuthorityBump;
    console.log("vaultAuthorityPda", vaultAuthorityPda);
    console.log("vaultAuthorityBump", vaultAuthorityBump);

    console.log("start initialize");

    // initializerはtoken accountとbump takerは直接initializerに払い出すのでtoken accountのみ
    // writebleである必要があるかどうかは不明？
    const remainingAccounts = [];
    remainingAccounts.push({
      pubkey: initializerTokenAccountA.address,
      isWritable: true, // falseにすると Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account

      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaA, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: true, // falseにすると Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account

      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintA, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountB.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaB, //vaultAccountB.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintB, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountC.address,
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountD.address,
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintD, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountE.address,
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintE, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });

    await program.rpc.initialize(
      new anchor.BN(initializerAdditionalSolAmount),
      new anchor.BN(takerAdditionalSolAmount),
      initializerNftAmount,
      takerNftAmount,
      Buffer.from(vaultAccountBumps), // 難関　Buffer.fromしないとTypeError: Blob.encode[data] requires (length 2) Buffer as src
      vaultAuthorityBump,
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          taker: takerMainAccount.publicKey,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          vaultAuthority: vaultAuthorityPda
        },

        remainingAccounts,
        signers: [
          escrowAccount, // rust側でinitするために必要　escrowAccount抜かすとError: Signature verification failed
          initializerMainAccount,
        ],
      }
    );

    console.log("start assertion");

    // Check token accounts
    let _initializerTokenAccountA = await getAccount(
      provider.connection,
      initializerTokenAccountA.address
    );
    assert.ok(Number(_initializerTokenAccountA.amount) === 0);

    let _initializerTokenAccountB = await getAccount(
      provider.connection,
      initializerTokenAccountB.address
    );
    assert.ok(Number(_initializerTokenAccountB.amount) === 0);

    let _initializerTokenAccountC = await getAccount(
      provider.connection,
      initializerTokenAccountC.address
    );
    assert.ok(Number(_initializerTokenAccountC.amount) === 0);

    let _initializerTokenAccountD = await getAccount(
      provider.connection,
      initializerTokenAccountD.address
    );
    assert.ok(Number(_initializerTokenAccountD.amount) === 0);

    let _initializerTokenAccountE = await getAccount(
      provider.connection,
      initializerTokenAccountE.address
    );
    assert.ok(Number(_initializerTokenAccountE.amount) === 0);

    let _takerTokenAccountA = await getAccount(
      provider.connection,
      takerTokenAccountA.address
    );
    assert.ok(Number(_takerTokenAccountA.amount) === 0);

    let _takerTokenAccountB = await getAccount(
      provider.connection,
      takerTokenAccountB.address
    );
    assert.ok(Number(_takerTokenAccountB.amount) === 0);

    let _takerTokenAccountC = await getAccount(
      provider.connection,
      takerTokenAccountC.address
    );
    assert.ok(Number(_takerTokenAccountC.amount) === 1);

    let _takerTokenAccountD = await getAccount(
      provider.connection,
      takerTokenAccountD.address
    );
    assert.ok(Number(_takerTokenAccountD.amount) === 1);

    let _takerTokenAccountE = await getAccount(
      provider.connection,
      takerTokenAccountE.address
    );
    assert.ok(Number(_takerTokenAccountE.amount) === 1);

    // Check vault
    const _vaultA = await provider.connection.getParsedAccountInfo(
      vaultAccountPdaA
    );
    const _vaultParsedDataA = _vaultA.value
      .data as anchor.web3.ParsedAccountData;
    assert.ok(_vaultA.value.owner.equals(TOKEN_PROGRAM_ID));
    assert.ok(
      new PublicKey(_vaultParsedDataA.parsed.info.owner).equals(
        vaultAuthorityPda
      )
    );
    assert.ok(Number(_vaultParsedDataA.parsed.info.tokenAmount.amount) === 1);

    const _vaultB = await provider.connection.getParsedAccountInfo(
      vaultAccountPdaB
    );
    const _vaultParsedDataB = _vaultB.value
      .data as anchor.web3.ParsedAccountData;
    assert.ok(_vaultB.value.owner.equals(TOKEN_PROGRAM_ID));
    assert.ok(
      new PublicKey(_vaultParsedDataB.parsed.info.owner).equals(
        vaultAuthorityPda
      )
    );
    assert.ok(Number(_vaultParsedDataB.parsed.info.tokenAmount.amount) === 1);

    // Check the escrowAccount
    let _escrowAccount = await program.account.escrowAccount.fetch(
      escrowAccount.publicKey
    );

    assert.ok(
      _escrowAccount.initializerKey.equals(initializerMainAccount.publicKey)
    );
    assert.ok(
      _escrowAccount.initializerAdditionalSolAmount.toNumber() ===
        initializerAdditionalSolAmount
    );
    assert.ok(
      _escrowAccount.initializerNftTokenAccounts.length === initializerNftAmount
    );
    const initializerNftTokenAccountAddresses =
      _escrowAccount.initializerNftTokenAccounts.map(
        (initializerNftTokenAccount) => initializerNftTokenAccount.toBase58()
      );
    assert.include(
      initializerNftTokenAccountAddresses,
      initializerTokenAccountA.address.toBase58()
    );
    assert.include(
      initializerNftTokenAccountAddresses,
      initializerTokenAccountB.address.toBase58()
    );
    assert.notInclude(
      initializerNftTokenAccountAddresses,
      initializerTokenAccountC.address.toBase58()
    );
    assert.notInclude(
      initializerNftTokenAccountAddresses,
      initializerTokenAccountD.address.toBase58()
    );
    assert.notInclude(
      initializerNftTokenAccountAddresses,
      initializerTokenAccountE.address.toBase58()
    );

    assert.ok(_escrowAccount.takerKey.equals(takerMainAccount.publicKey));
    assert.ok(
      _escrowAccount.takerAdditionalSolAmount.toNumber() ===
        takerAdditionalSolAmount
    );
    assert.ok(_escrowAccount.takerNftTokenAccounts.length === takerNftAmount);
    const takerNftTokenAccountAddresses =
      _escrowAccount.takerNftTokenAccounts.map((takerNftTokenAccount) =>
        takerNftTokenAccount.toBase58()
      );
    assert.notInclude(
      takerNftTokenAccountAddresses,
      takerTokenAccountA.address.toBase58()
    );
    assert.notInclude(
      takerNftTokenAccountAddresses,
      takerTokenAccountB.address.toBase58()
    );
    assert.include(
      takerNftTokenAccountAddresses,
      takerTokenAccountC.address.toBase58()
    );
    assert.include(
      takerNftTokenAccountAddresses,
      takerTokenAccountD.address.toBase58()
    );
    assert.include(
      takerNftTokenAccountAddresses,
      takerTokenAccountE.address.toBase58()
    );

    assert.ok(
      (_escrowAccount.vaultAccountBumps as Buffer).equals(
        Buffer.from(vaultAccountBumps)
      )
    );

    // SOLの移動検証
    const _escrowAccountInfo = await provider.connection.getParsedAccountInfo(
      escrowAccount.publicKey
    );
    assert.ok(
      _escrowAccountInfo.value.lamports >= initializerAdditionalSolAmount
    );

    // 連続してinitializeしたときのテスト
    // 同じ組み合わせのinitializer, takerで二重に取引できない
  });

  it("Exchange escrow state", async () => {
    console.log("vaultAccountPdaA", vaultAccountPdaA);
    console.log("vaultAccountPdaB", vaultAccountPdaB);

    const [_vaultAuthorityPda, _vaultAuthorityBump] =
    await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("vault-authority"),
        initializerMainAccount.publicKey.toBuffer(),
        takerMainAccount.publicKey.toBuffer(),
      ], // sampleコードではBufferが書いてあったがBufferはいらないと思われる anchor bookにはない
      program.programId
    );
    vaultAuthorityPda = _vaultAuthorityPda;
    vaultAuthorityBump = _vaultAuthorityBump;
    console.log("vaultAuthorityPda", vaultAuthorityPda);
    console.log("vaultAuthorityBump", vaultAuthorityBump);

    const remainingAccounts = [];
    remainingAccounts.push({
      pubkey: initializerTokenAccountA.address,
      isWritable: false, // falseでOK
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaA,
      isWritable: true, // trueでないと駄目
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintA, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountB.address,
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaB,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintB, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountC.address,
      isWritable: true, // false だと駄目
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountD.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintD, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountE.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintE, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountA.address,
      isWritable: true, // falseだと駄目
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintA, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountB.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintB, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountC.address,
      isWritable: true, // falseだと駄目
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountD.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintD, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountE.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintE, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    console.log("initializerMainAccount", initializerMainAccount);
    console.log(
      "initializerMainAccount.publicKey",
      initializerMainAccount.publicKey
    );
    console.log("takerMainAccount", takerMainAccount);
    console.log("takerMainAccount.publicKey", takerMainAccount.publicKey);
    console.log("vaultAuthorityPda", vaultAuthorityPda);

    /* escrowに入っているお金を抜けないかテスト
      programでやると抜ける
      tsでやろうとすると次のエラー Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: invalid program argument
      */
    try {
      await provider.send(
        (() => {
          const tx = new Transaction();
          tx.add(
            SystemProgram.transfer({
              fromPubkey: escrowAccount.publicKey,
              toPubkey: takerMainAccount.publicKey,
              lamports: initializerAdditionalSolAmount,
            })
          );
          return tx;
        })(),
        [escrowAccount]
      );
    } catch (err) {
      assert.ok(err !== null);
    }

    const beforeInitializerAccounts =
      await provider.connection.getParsedTokenAccountsByOwner(
        initializerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );

    console.log(
      "initializer main account",
      initializerMainAccount.publicKey.toBase58() // TODO: これで所有者のチェック可能
    );
    beforeInitializerAccounts.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("initializer", account.data.parsed.info);
      console.log("initializer mint", mint);
      console.log("initializer tokenAmount", tokenAmount);
    });

    const beforeTakerTokens =
      await provider.connection.getParsedTokenAccountsByOwner(
        takerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );

    console.log("taker main account", takerMainAccount.publicKey);
    beforeTakerTokens.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("taker", account.data.parsed.info);
      console.log("taker mint", mint);
      console.log("taker tokenAmount before", tokenAmount);
    });

    await program.rpc.exchange(
      new anchor.BN(initializerAdditionalSolAmount), // この変数がないとaccountsが読めず、taker not providedエラーが生じる
      new anchor.BN(takerAdditionalSolAmount),
      vaultAuthorityBump,
      {
        accounts: {
          taker: takerMainAccount.publicKey,
          // vaultSolAccount: vaultSolAccountPda, // ここの値が違うと Error: 3012: The program expected this account to be already initialized
          initializer: initializerMainAccount.publicKey,
          escrowAccount: escrowAccount.publicKey,
          vaultAuthority: vaultAuthorityPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        remainingAccounts,
        signers: [takerMainAccount],
      }
    );

    console.log("start assertion");

    // NFTの数の検証
    let _initializerTokenAccountA = await getAccount(
      provider.connection,
      initializerTokenAccountA.address
    );
    assert.ok(Number(_initializerTokenAccountA.amount) === 0);

    let _initializerTokenAccountB = await getAccount(
      provider.connection,
      initializerTokenAccountB.address
    );
    assert.ok(Number(_initializerTokenAccountB.amount) === 0);

    let _initializerTokenAccountC = await getAccount(
      provider.connection,
      initializerTokenAccountC.address
    );
    assert.ok(Number(_initializerTokenAccountC.amount) === 1);

    let _initializerTokenAccountD = await getAccount(
      provider.connection,
      initializerTokenAccountD.address
    );
    assert.ok(Number(_initializerTokenAccountD.amount) === 1);

    let _initializerTokenAccountE = await getAccount(
      provider.connection,
      initializerTokenAccountE.address
    );
    assert.ok(Number(_initializerTokenAccountE.amount) === 1);

    let _takerTokenAccountA = await getAccount(
      provider.connection,
      takerTokenAccountA.address
    );
    assert.ok(Number(_takerTokenAccountA.amount) === 1);

    let _takerTokenAccountB = await getAccount(
      provider.connection,
      takerTokenAccountB.address
    );
    assert.ok(Number(_takerTokenAccountB.amount) === 1);

    let _takerTokenAccountC = await getAccount(
      provider.connection,
      takerTokenAccountC.address
    );
    assert.ok(Number(_takerTokenAccountC.amount) === 0);

    let _takerTokenAccountD = await getAccount(
      provider.connection,
      takerTokenAccountD.address
    );
    assert.ok(Number(_takerTokenAccountD.amount) === 0);

    let _takerTokenAccountE = await getAccount(
      provider.connection,
      takerTokenAccountE.address
    );
    assert.ok(Number(_takerTokenAccountE.amount) === 0);

    // initializerで署名してもvaultからsol引き落とそうとしたらError: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account
    // のエラーがでるが、テストで保証しておきたい

    // 追加SOLのチェック token closeしないとずれる
    // close = initializerありなしでもずれる
    /* close = initializer入れてるから以下は取得できない
    let _escrowAccount2 = await provider.connection.getAccountInfo(
      escrowAccount.publicKey
    );
    console.log("_escrowAccount2.lamports", _escrowAccount2.lamports);
    */
    const initializer = await provider.connection.getAccountInfo(
      initializerMainAccount.publicKey
    );
    console.log("initializer.lamports", initializer.lamports);
    console.log("initializerStartSolAmount", initializerStartSolAmount);
    console.log(
      "initializerAdditionalSolAmount",
      initializerAdditionalSolAmount
    );

    const afterInitializerAccounts =
      await provider.connection.getParsedTokenAccountsByOwner(
        initializerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    afterInitializerAccounts.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("initializer mint", mint);
      console.log("initializer tokenAmount", tokenAmount);
    });

    const afterTakerTokens =
      await provider.connection.getParsedTokenAccountsByOwner(
        takerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    afterTakerTokens.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("taker mint", mint);
      console.log("taker tokenAmount", tokenAmount);
    });

    // Check vault
    const _vaultA = await provider.connection.getParsedAccountInfo(
      vaultAccountPdaA
    );
    assert.ok(_vaultA.value === null);

    const _vaultB = await provider.connection.getParsedAccountInfo(
      vaultAccountPdaB
    );
    assert.ok(_vaultB.value === null);

    // check escrow account
    try {
      await program.account.escrowAccount.fetch(escrowAccount.publicKey);
    } catch (err) {
      assert.ok(err !== null);
    }

    // check escrow authority
    const _vaultAuthority = await provider.connection.getParsedAccountInfo(
      vaultAuthorityPda
    );
    assert.ok(_vaultAuthority.value === null);

    // SOLの移動検証
    const _initializerMainAccountInfo =
      await provider.connection.getParsedAccountInfo(
        initializerMainAccount.publicKey
      );
    console.log(
      "_initializerMainAccountInfo.value.lamports",
      _initializerMainAccountInfo.value.lamports
    );
    console.log("initializerStartSolAmount", initializerStartSolAmount);

    const _takerMainAccountInfo =
      await provider.connection.getParsedAccountInfo(
        takerMainAccount.publicKey
      );
    console.log(
      "_takerMainAccountInfo.value.lamports",
      _takerMainAccountInfo.value.lamports
    );
    console.log("takerStartSolAmount", takerStartSolAmount);

    assert.ok(
      _initializerMainAccountInfo.value.lamports >= initializerStartSolAmount
    );
  });

  it("Initialize escrow and cancel escrow by A", async () => {
    const [_vaultAuthorityPda, _vaultAuthorityBump] =
    await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("vault-authority"),
        initializerMainAccount.publicKey.toBuffer(),
        takerMainAccount.publicKey.toBuffer(),
      ], // sampleコードではBufferが書いてあったがBufferはいらないと思われる anchor bookにはない
      program.programId
    );
    vaultAuthorityPda = _vaultAuthorityPda;
    vaultAuthorityBump = _vaultAuthorityBump;
    console.log("vaultAuthorityPda", vaultAuthorityPda);
    console.log("vaultAuthorityBump", vaultAuthorityBump);

    // exchangeした直後なのでinitializerがC D EのNFT takerがA BのNFTを持っている
    const [_vaultAccountPdaC, _vaultAccountBumpC] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountC.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaC = _vaultAccountPdaC;
    vaultAccountBumpC = _vaultAccountBumpC;
    console.log("initializerTokenAccountC", initializerTokenAccountC);
    console.log("vaultAccountPdaC", vaultAccountPdaC);
    console.log("vaultAccountBumpC", vaultAccountBumpC);

    const [_vaultAccountPdaD, _vaultAccountBumpD] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountD.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaD = _vaultAccountPdaD;
    vaultAccountBumpD = _vaultAccountBumpD;
    console.log("initializerTokenAccountD", initializerTokenAccountD);
    console.log("vaultAccountPdaD", vaultAccountPdaD);
    console.log("vaultAccountBumpD", vaultAccountBumpD);

    const [_vaultAccountPdaE, _vaultAccountBumpE] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountE.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaE = _vaultAccountPdaE;
    vaultAccountBumpE = _vaultAccountBumpE;
    console.log("initializerTokenAccountE", initializerTokenAccountE);
    console.log("vaultAccountPdaE", vaultAccountPdaE);
    console.log("vaultAccountBumpE", vaultAccountBumpE);

    const vaultAccountBumps: number[] = [
      vaultAccountBumpC,
      vaultAccountBumpD,
      vaultAccountBumpE,
    ];
    console.log("vaultAccountBumps", vaultAccountBumps);

    let remainingAccounts = [];
    remainingAccounts.push({
      pubkey: initializerTokenAccountC.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountD.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaD, //vaultAccountB.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintD, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountE.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaE, //vaultAccountB.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintE, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountA.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintA, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountB.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintB, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });

    initializerNftAmount = 3;
    takerNftAmount = 2;

    await program.rpc.initialize(
      new anchor.BN(initializerAdditionalSolAmount),
      new anchor.BN(takerAdditionalSolAmount),
      initializerNftAmount,
      takerNftAmount,
      Buffer.from(vaultAccountBumps), // 難関　Buffer.fromしないとTypeError: Blob.encode[data] requires (length 2) Buffer as src
      vaultAuthorityBump,
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          taker: takerMainAccount.publicKey,
          // vaultAccount: vaultAccountPda,
          // vaultSolAccount: vaultSolAccountPda,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          vaultAuthority: vaultAuthorityPda
        },
        instructions: [
          // await program.account.escrowAccount.createInstruction(escrowAccount), // 抜かすとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: Program failed to complete
          // createTempTokenAccountAIx, RUSTに移管
          //initTempAccountAIx,
          //createTempTokenAccountBIx,
          // initTempAccountBIx,
        ],
        remainingAccounts: remainingAccounts,
        signers: [
          escrowAccount,
          initializerMainAccount,
          // vaultAccountA,
          // vaultAccountB,
        ], // escrowAccount抜かすとError: Signature verification failed
      }
    );
    let _escrowAccountInfo = await provider.connection.getAccountInfo(
      escrowAccount.publicKey
    );
    console.log("_escrowAccountInfo.lamports", _escrowAccountInfo.lamports);

    // NFTの数の検証
    let _initializerTokenAccountA = await getAccount(
      provider.connection,
      initializerTokenAccountA.address
    );
    assert.ok(Number(_initializerTokenAccountA.amount) === 0);

    let _initializerTokenAccountB = await getAccount(
      provider.connection,
      initializerTokenAccountB.address
    );
    assert.ok(Number(_initializerTokenAccountB.amount) === 0);

    let _initializerTokenAccountC = await getAccount(
      provider.connection,
      initializerTokenAccountC.address
    );
    assert.ok(Number(_initializerTokenAccountC.amount) === 0);

    let _initializerTokenAccountD = await getAccount(
      provider.connection,
      initializerTokenAccountD.address
    );
    assert.ok(Number(_initializerTokenAccountD.amount) === 0);

    let _initializerTokenAccountE = await getAccount(
      provider.connection,
      initializerTokenAccountE.address
    );
    assert.ok(Number(_initializerTokenAccountE.amount) === 0);

    let _takerTokenAccountA = await getAccount(
      provider.connection,
      takerTokenAccountA.address
    );
    assert.ok(Number(_takerTokenAccountA.amount) === 1);

    let _takerTokenAccountB = await getAccount(
      provider.connection,
      takerTokenAccountB.address
    );
    assert.ok(Number(_takerTokenAccountB.amount) === 1);

    let _takerTokenAccountC = await getAccount(
      provider.connection,
      takerTokenAccountC.address
    );
    assert.ok(Number(_takerTokenAccountC.amount) === 0);

    let _takerTokenAccountD = await getAccount(
      provider.connection,
      takerTokenAccountD.address
    );
    assert.ok(Number(_takerTokenAccountD.amount) === 0);

    let _takerTokenAccountE = await getAccount(
      provider.connection,
      takerTokenAccountE.address
    );
    assert.ok(Number(_takerTokenAccountE.amount) === 0);

    // 以下からが実際のcancel時に必要な処理
    // remaining accountsの構造　initializerの返却するNFTのみ 3で割った mod0がtoken account mod1がvault account mod2がmint
    console.log("start cancel");
    remainingAccounts = [];
    remainingAccounts.push({
      pubkey: initializerTokenAccountC.address,
      isWritable: true, // falseだと駄目
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: true, // falseだと駄目
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountD.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaD, //vaultAccountB.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintD, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountE.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaE, //vaultAccountB.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintE, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });

    const beforeInitializerAccounts =
      await provider.connection.getParsedTokenAccountsByOwner(
        initializerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    beforeInitializerAccounts.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("initializer mint", mint);
      console.log("initializer tokenAmount", tokenAmount);
    });

    const beforeTakerTokens =
      await provider.connection.getParsedTokenAccountsByOwner(
        takerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    beforeTakerTokens.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("taker mint", mint);
      console.log("taker tokenAmount", tokenAmount);
    });

    // Cancel the escrow.
    await program.rpc.cancelByInitializer(
      // new anchor.BN(initializerAdditionalSolAmount),
      vaultAuthorityBump,
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          taker: takerMainAccount.publicKey,
          vaultAuthority: vaultAuthorityPda,
          escrowAccount: escrowAccount.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [initializerMainAccount],
        remainingAccounts,
      }
    );

    const afterInitializerAccounts =
      await provider.connection.getParsedTokenAccountsByOwner(
        initializerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    afterInitializerAccounts.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("initializer mint", mint);
      console.log("initializer tokenAmount", tokenAmount);
    });

    const afterTakerTokens =
      await provider.connection.getParsedTokenAccountsByOwner(
        takerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    afterTakerTokens.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("taker mint", mint);
      console.log("taker tokenAmount", tokenAmount);
    });

    // NFTの数の検証
    _initializerTokenAccountA = await getAccount(
      provider.connection,
      initializerTokenAccountA.address
    );
    assert.ok(Number(_initializerTokenAccountA.amount) === 0);

    _initializerTokenAccountB = await getAccount(
      provider.connection,
      initializerTokenAccountB.address
    );
    assert.ok(Number(_initializerTokenAccountB.amount) === 0);

    _initializerTokenAccountC = await getAccount(
      provider.connection,
      initializerTokenAccountC.address
    );
    assert.ok(Number(_initializerTokenAccountC.amount) === 1);

    _initializerTokenAccountD = await getAccount(
      provider.connection,
      initializerTokenAccountD.address
    );
    assert.ok(Number(_initializerTokenAccountD.amount) === 1);

    _initializerTokenAccountE = await getAccount(
      provider.connection,
      initializerTokenAccountE.address
    );
    assert.ok(Number(_initializerTokenAccountE.amount) === 1);

    _takerTokenAccountA = await getAccount(
      provider.connection,
      takerTokenAccountA.address
    );
    assert.ok(Number(_takerTokenAccountA.amount) === 1);

    _takerTokenAccountB = await getAccount(
      provider.connection,
      takerTokenAccountB.address
    );
    assert.ok(Number(_takerTokenAccountB.amount) === 1);

    _takerTokenAccountC = await getAccount(
      provider.connection,
      takerTokenAccountC.address
    );
    assert.ok(Number(_takerTokenAccountC.amount) === 0);

    _takerTokenAccountD = await getAccount(
      provider.connection,
      takerTokenAccountD.address
    );
    assert.ok(Number(_takerTokenAccountD.amount) === 0);

    _takerTokenAccountE = await getAccount(
      provider.connection,
      takerTokenAccountE.address
    );
    assert.ok(Number(_takerTokenAccountE.amount) === 0);

    // Check vault
    const _vaultA = await provider.connection.getParsedAccountInfo(
      vaultAccountPdaA
    );
    assert.ok(_vaultA.value === null);

    const _vaultB = await provider.connection.getParsedAccountInfo(
      vaultAccountPdaB
    );
    assert.ok(_vaultB.value === null);

    // check escrow account
    try {
      await program.account.escrowAccount.fetch(escrowAccount.publicKey);
    } catch (err) {
      assert.ok(err !== null);
    }

    // check escrow authority
    const _vaultAuthority = await provider.connection.getParsedAccountInfo(
      vaultAuthorityPda
    );
    assert.ok(_vaultAuthority.value === null);
  });

  it("Initialize escrow and cancel escrow by B", async () => {
    const [_vaultAuthorityPda, _vaultAuthorityBump] =
    await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("vault-authority"),
        initializerMainAccount.publicKey.toBuffer(),
        takerMainAccount.publicKey.toBuffer(),
      ], // sampleコードではBufferが書いてあったがBufferはいらないと思われる anchor bookにはない
      program.programId
    );
    vaultAuthorityPda = _vaultAuthorityPda;
    vaultAuthorityBump = _vaultAuthorityBump;
    console.log("vaultAuthorityPda", vaultAuthorityPda);
    console.log("vaultAuthorityBump", vaultAuthorityBump);

    const [_vaultAccountPdaC, _vaultAccountBumpC] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountC.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaC = _vaultAccountPdaC;
    vaultAccountBumpC = _vaultAccountBumpC;
    console.log("initializerTokenAccountC", initializerTokenAccountC);
    console.log("vaultAccountPdaC", vaultAccountPdaC);
    console.log("vaultAccountBumpC", vaultAccountBumpC);

    const [_vaultAccountPdaD, _vaultAccountBumpD] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountD.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaD = _vaultAccountPdaD;
    vaultAccountBumpD = _vaultAccountBumpD;
    console.log("initializerTokenAccountD", initializerTokenAccountD);
    console.log("vaultAccountPdaD", vaultAccountPdaD);
    console.log("vaultAccountBumpD", vaultAccountBumpD);

    const [_vaultAccountPdaE, _vaultAccountBumpE] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("vault-account")),
          initializerTokenAccountE.address.toBuffer(),
        ],
        program.programId
      );
    vaultAccountPdaE = _vaultAccountPdaE;
    vaultAccountBumpE = _vaultAccountBumpE;
    console.log("initializerTokenAccountE", initializerTokenAccountE);
    console.log("vaultAccountPdaE", vaultAccountPdaE);
    console.log("vaultAccountBumpE", vaultAccountBumpE);

    const vaultAccountBumps: number[] = [
      vaultAccountBumpC,
      vaultAccountBumpD,
      vaultAccountBumpE,
    ];
    console.log("vaultAccountBumps", vaultAccountBumps);

    let remainingAccounts = [];
    remainingAccounts.push({
      pubkey: initializerTokenAccountC.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountD.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaD, //vaultAccountB.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintD, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountE.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaE, //vaultAccountB.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintE, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountA.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintA, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: takerTokenAccountB.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintB, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });

    await program.rpc.initialize(
      new anchor.BN(initializerAdditionalSolAmount),
      new anchor.BN(takerAdditionalSolAmount),
      initializerNftAmount,
      takerNftAmount,
      Buffer.from(vaultAccountBumps), // 難関　Buffer.fromしないとTypeError: Blob.encode[data] requires (length 2) Buffer as src
      vaultAuthorityBump,
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          taker: takerMainAccount.publicKey,
          // vaultAccount: vaultAccountPda,
          // vaultSolAccount: vaultSolAccountPda,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          vaultAuthority: vaultAuthorityPda,
        },
        instructions: [
          // await program.account.escrowAccount.createInstruction(escrowAccount), // 抜かすとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: Program failed to complete
          // createTempTokenAccountAIx, RUSTに移管
          //initTempAccountAIx,
          //createTempTokenAccountBIx,
          // initTempAccountBIx,
        ],
        remainingAccounts: remainingAccounts,
        signers: [
          escrowAccount,
          initializerMainAccount,
          // vaultAccountA,
          // vaultAccountB,
        ], // escrowAccount抜かすとError: Signature verification failed
      }
    );
    let _escrowAccountInfo = await provider.connection.getAccountInfo(
      escrowAccount.publicKey
    );
    console.log("_escrowAccountInfo.lamports", _escrowAccountInfo.lamports);

    // NFTの数の検証
    let _initializerTokenAccountA = await getAccount(
      provider.connection,
      initializerTokenAccountA.address
    );
    assert.ok(Number(_initializerTokenAccountA.amount) === 0);

    let _initializerTokenAccountB = await getAccount(
      provider.connection,
      initializerTokenAccountB.address
    );
    assert.ok(Number(_initializerTokenAccountB.amount) === 0);

    let _initializerTokenAccountC = await getAccount(
      provider.connection,
      initializerTokenAccountC.address
    );
    assert.ok(Number(_initializerTokenAccountC.amount) === 0);

    let _initializerTokenAccountD = await getAccount(
      provider.connection,
      initializerTokenAccountD.address
    );
    assert.ok(Number(_initializerTokenAccountD.amount) === 0);

    let _initializerTokenAccountE = await getAccount(
      provider.connection,
      initializerTokenAccountE.address
    );
    assert.ok(Number(_initializerTokenAccountE.amount) === 0);

    let _takerTokenAccountA = await getAccount(
      provider.connection,
      takerTokenAccountA.address
    );
    assert.ok(Number(_takerTokenAccountA.amount) === 1);

    let _takerTokenAccountB = await getAccount(
      provider.connection,
      takerTokenAccountB.address
    );
    assert.ok(Number(_takerTokenAccountB.amount) === 1);

    let _takerTokenAccountC = await getAccount(
      provider.connection,
      takerTokenAccountC.address
    );
    assert.ok(Number(_takerTokenAccountC.amount) === 0);

    let _takerTokenAccountD = await getAccount(
      provider.connection,
      takerTokenAccountD.address
    );
    assert.ok(Number(_takerTokenAccountD.amount) === 0);

    let _takerTokenAccountE = await getAccount(
      provider.connection,
      takerTokenAccountE.address
    );
    assert.ok(Number(_takerTokenAccountE.amount) === 0);

    // 以下からが実際のcancel時に必要な処理
    // remaining accountsの構造　initializerの返却するNFTのみ 3で割った mod0がtoken account mod1がvault account mod2がmint
    console.log("start cancel");
    remainingAccounts = [];
    remainingAccounts.push({
      pubkey: initializerTokenAccountC.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintC, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountD.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaD, //vaultAccountB.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintD, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: initializerTokenAccountE.address,
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: vaultAccountPdaE, //vaultAccountB.publicKeyでも動くがPDAに移管
      isWritable: true,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: mintE, //vaultAccountA.publicKeyでも動くがPDAに移管
      isWritable: false,
      isSigner: false,
    });

    const beforeInitializerAccounts =
      await provider.connection.getParsedTokenAccountsByOwner(
        initializerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    beforeInitializerAccounts.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("initializer mint", mint);
      console.log("initializer tokenAmount", tokenAmount);
    });

    const beforeTakerTokens =
      await provider.connection.getParsedTokenAccountsByOwner(
        takerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    beforeTakerTokens.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("taker mint", mint);
      console.log("taker tokenAmount", tokenAmount);
    });

    // Cancel the escrow.
    await program.rpc.cancelByTaker(
      // new anchor.BN(initializerAdditionalSolAmount),
      vaultAuthorityBump,
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          taker: takerMainAccount.publicKey,
          vaultAuthority: vaultAuthorityPda,
          escrowAccount: escrowAccount.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [takerMainAccount],
        remainingAccounts,
      }
    );

    const afterInitializerAccounts =
      await provider.connection.getParsedTokenAccountsByOwner(
        initializerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    afterInitializerAccounts.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("initializer mint", mint);
      console.log("initializer tokenAmount", tokenAmount);
    });

    const afterTakerTokens =
      await provider.connection.getParsedTokenAccountsByOwner(
        takerMainAccount.publicKey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    afterTakerTokens.value.map(({ account }) => {
      const { mint, tokenAmount } = account.data.parsed.info;
      console.log("taker mint", mint);
      console.log("taker tokenAmount", tokenAmount);
    });

    // NFTの数の検証
    _initializerTokenAccountA = await getAccount(
      provider.connection,
      initializerTokenAccountA.address
    );
    assert.ok(Number(_initializerTokenAccountA.amount) === 0);

    _initializerTokenAccountB = await getAccount(
      provider.connection,
      initializerTokenAccountB.address
    );
    assert.ok(Number(_initializerTokenAccountB.amount) === 0);

    _initializerTokenAccountC = await getAccount(
      provider.connection,
      initializerTokenAccountC.address
    );
    assert.ok(Number(_initializerTokenAccountC.amount) === 1);

    _initializerTokenAccountD = await getAccount(
      provider.connection,
      initializerTokenAccountD.address
    );
    assert.ok(Number(_initializerTokenAccountD.amount) === 1);

    _initializerTokenAccountE = await getAccount(
      provider.connection,
      initializerTokenAccountE.address
    );
    assert.ok(Number(_initializerTokenAccountE.amount) === 1);

    _takerTokenAccountA = await getAccount(
      provider.connection,
      takerTokenAccountA.address
    );
    assert.ok(Number(_takerTokenAccountA.amount) === 1);

    _takerTokenAccountB = await getAccount(
      provider.connection,
      takerTokenAccountB.address
    );
    assert.ok(Number(_takerTokenAccountB.amount) === 1);

    _takerTokenAccountC = await getAccount(
      provider.connection,
      takerTokenAccountC.address
    );
    assert.ok(Number(_takerTokenAccountC.amount) === 0);

    _takerTokenAccountD = await getAccount(
      provider.connection,
      takerTokenAccountD.address
    );
    assert.ok(Number(_takerTokenAccountD.amount) === 0);

    _takerTokenAccountE = await getAccount(
      provider.connection,
      takerTokenAccountE.address
    );
    assert.ok(Number(_takerTokenAccountE.amount) === 0);

    // Check vault
    const _vaultA = await provider.connection.getParsedAccountInfo(
      vaultAccountPdaA
    );
    assert.ok(_vaultA.value === null);

    const _vaultB = await provider.connection.getParsedAccountInfo(
      vaultAccountPdaB
    );
    assert.ok(_vaultB.value === null);

    // check escrow account
    try {
      await program.account.escrowAccount.fetch(escrowAccount.publicKey);
    } catch (err) {
      assert.ok(err !== null);
    }

    // check escrow authority
    const _vaultAuthority = await provider.connection.getParsedAccountInfo(
      vaultAuthorityPda
    );
    assert.ok(_vaultAuthority.value === null);
  });

  // initializerがSOL払う場合

  // takerがSOL払う場合
});
