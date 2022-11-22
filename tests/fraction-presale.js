const anchor = require('@project-serum/anchor');
const splToken = require('@solana/spl-token');
const BN = require('BN.js');
const { SystemProgram, SYSVAR_RENT_PUBKEY, Keypair, Transaction } = anchor.web3;
const { assert } = require('chai');


describe('fraction-presale', () => {

	let provider = anchor.Provider.env();
	const connection = provider.connection;
	const payer = provider.wallet.payer;
	const payerKey = payer.publicKey;
	const program = anchor.workspace.FractionPresale;
	anchor.setProvider(provider);

	const presaleAccount = anchor.web3.Keypair.generate();
	const fractionTreasury = anchor.web3.Keypair.generate();
	const paymentTreasury = anchor.web3.Keypair.generate();
	const accessTreasury = anchor.web3.Keypair.generate();
	const vestingAccount = anchor.web3.Keypair.generate();

	var presalePDA;
	var presalePDABump;

	
	const price = 0.1;
	const maxAmount = 1_000;
	const totalFractions = 1_000_000;
	var startTimestamp = 0;
	const DECIMALS = 9;

	var fractionMint;
	var paymentMint;
	var accessMint;
	var payerAccessAccount;
	var payerFractionAccount;

	const getCurrentTimestamp = (n=0) => {
		return n + Math.floor(Date.now() / 1000);
	}

	const presaleEnd = new BN(getCurrentTimestamp(1000));
	const vestingEnd = new BN(getCurrentTimestamp(1000));

	const createNativeTokenAccount = async (amount, user=payer) => {
		let balanceNeeded = await splToken.Token.getMinBalanceRentForExemptAccount(connection);
		let newAccount = Keypair.generate();
		let transaction = new Transaction();
		transaction.add(
			SystemProgram.createAccount({
				fromPubkey: payer.publicKey,
				newAccountPubkey: newAccount.publicKey,
				lamports: balanceNeeded + (amount * 1e9),
				space: splToken.AccountLayout.span,
				programId: splToken.TOKEN_PROGRAM_ID,
			}),
			splToken.Token.createInitAccountInstruction(
				splToken.TOKEN_PROGRAM_ID,
				splToken.NATIVE_MINT,
				newAccount.publicKey,
				user.publicKey,
			)
		);
		await provider.send(transaction, [payer, newAccount]);
		return newAccount.publicKey;
	}

	const createMint = async (authority=payerKey) => {
		return await splToken.Token.createMint(
			provider.connection, payer, 
			authority, // mint authority
			authority, // freeze authority
			DECIMALS, splToken.TOKEN_PROGRAM_ID,
		);
	}

	const getAccessTokens = async (toAccount, amount) => {
		await accessMint.mintTo(toAccount, payerKey, [], amount);
	}

	const getTokenAccountBalance = async (address) => {
		let res = await connection.getTokenAccountBalance(address);
		return new BN(res.value.amount);
	}

	// Add fractions to the presale and check that the token account balances changed as expected
	const addFractionsToPresale = async (amount, fromAccount, user=payer) => {
		let fromBalanceBefore = await getTokenAccountBalance(fromAccount);
		let fractionBalanceBefore = await getTokenAccountBalance(fractionTreasury.publicKey);

		amount = new BN((amount * 10**DECIMALS).toString())
		await program.rpc.addFractionsForSale(presalePDABump, new BN(amount), {
			accounts: {
				presaleAccount: presaleAccount.publicKey,
				fractionTreasury: fractionTreasury.publicKey,
				paymentTreasury: paymentTreasury.publicKey,
				fromAccount: fromAccount,
				presalePda: presalePDA,
				authority: user.publicKey,
				tokenProgram: splToken.TOKEN_PROGRAM_ID,
			},
			signers: [user]
		});

		let fromBalanceAfter = await getTokenAccountBalance(fromAccount);
		let fractionBalanceAfter = await getTokenAccountBalance(fractionTreasury.publicKey);
		assert.equal(fromBalanceBefore.sub(fromBalanceAfter).toString(), amount.toString());
		assert.equal(fractionBalanceAfter.sub(fractionBalanceBefore).toString(), amount.toString());
	}
	
	// Remove fractions from the presale and check that the token account balances changed as expected
	const removeFractionsFromPresale = async (amount, toAccount, user=payer) => {
		let toBalanceBefore = await getTokenAccountBalance(toAccount);
		let fractionBalanceBefore = await getTokenAccountBalance(fractionTreasury.publicKey);

		amount = new BN((amount * 10**DECIMALS).toString())
		await program.rpc.removeFractionsForSale(presalePDABump, amount, {
			accounts: {
				presaleAccount: presaleAccount.publicKey,
				fractionTreasury: fractionTreasury.publicKey,
				paymentTreasury: paymentTreasury.publicKey,
				toAccount: toAccount,
				presalePda: presalePDA,
				authority: user.publicKey,
				tokenProgram: splToken.TOKEN_PROGRAM_ID,
			},
			signers: [user]
		});

		let toBalanceAfter = await getTokenAccountBalance(toAccount);
		let fractionBalanceAfter = await getTokenAccountBalance(fractionTreasury.publicKey);
		assert.equal(toBalanceAfter.sub(toBalanceBefore).toString(), amount.toString());
		assert.equal(fractionBalanceBefore.sub(fractionBalanceAfter).toString(), amount.toString());
	}

	const initVestingAccount = async (user=payer) => {
		let [userVestingPDA, userVestingPDABump] = await anchor.web3.PublicKey.findProgramAddress(
			[Buffer.from("vesting"), user.publicKey.toBuffer(), presaleAccount.publicKey.toBuffer(), program.programId.toBuffer()], 
			program.programId
		);
		await program.rpc.initVestingAccount(userVestingPDABump, {
			accounts: {
				presaleAccount: presaleAccount.publicKey,
				fractionTreasury: fractionTreasury.publicKey,
				paymentTreasury: paymentTreasury.publicKey,
				vestingAccount: vestingAccount.publicKey,
				userVestingPda: userVestingPDA,
				fractionMint: fractionMint.publicKey,
				signer: user.publicKey,
				tokenProgram: splToken.TOKEN_PROGRAM_ID,
				rent: SYSVAR_RENT_PUBKEY,
				systemProgram: SystemProgram.programId,
			},
			signers: [vestingAccount, user]
		});
	}

	const purchaseFractions = async (accessAccount, paymentAccount, amount, user=payer) => {
		let accessAccountBalanceBefore = await getTokenAccountBalance(accessAccount);
		let accessTreasuryBalanceBefore = await getTokenAccountBalance(accessTreasury.publicKey);
		let paymentAccountBalanceBefore = await getTokenAccountBalance(paymentAccount);
		let paymentTreasuryBalanceBefore = await getTokenAccountBalance(paymentTreasury.publicKey);

		let purchaseAmount = (amount * price * 10**DECIMALS).toString();
		amount = new BN((amount * 10**DECIMALS).toString());
		let [userVestingPDA, userVestingPDABump] = await anchor.web3.PublicKey.findProgramAddress(
			[Buffer.from("vesting"), user.publicKey.toBuffer(), presaleAccount.publicKey.toBuffer(), program.programId.toBuffer()], 
			program.programId
		);
		await program.rpc.purchaseFractions(presalePDABump, userVestingPDABump, amount, {
			accounts: {
				presaleAccount: presaleAccount.publicKey,
				fractionTreasury: fractionTreasury.publicKey,
				paymentTreasury: paymentTreasury.publicKey,
				accessTreasury: accessTreasury.publicKey,
				fromAccount: paymentAccount,
				vestingAccount: vestingAccount.publicKey,
				userVestingPda: userVestingPDA,
				presalePda: presalePDA,
				accessAccount: accessAccount,
				signer: user.publicKey,
				tokenProgram: splToken.TOKEN_PROGRAM_ID
			}
		});

		let accessAccountBalanceAfter = await getTokenAccountBalance(accessAccount);
		let accessTreasuryBalanceAfter = await getTokenAccountBalance(accessTreasury.publicKey);
		let paymentAccountBalanceAfter = await getTokenAccountBalance(paymentAccount);
		let paymentTreasuryBalanceAfter = await getTokenAccountBalance(paymentTreasury.publicKey);
		assert.equal(accessAccountBalanceBefore.sub(accessAccountBalanceAfter).toString(), '1');
		assert.equal(accessTreasuryBalanceAfter.sub(accessTreasuryBalanceBefore).toString(), '1');
		assert.equal(paymentAccountBalanceBefore.sub(paymentAccountBalanceAfter).toString(), purchaseAmount);
		assert.equal(paymentTreasuryBalanceAfter.sub(paymentTreasuryBalanceBefore).toString(), purchaseAmount);
	}

	const collectPayments = async (toAccount, user=payer) => {
		let toBalanceBefore = await getTokenAccountBalance(toAccount);
		let paymentBalanceBefore = await getTokenAccountBalance(paymentTreasury.publicKey);

		await program.rpc.collectFunds(presalePDABump, {
			accounts: {
				presaleAccount: presaleAccount.publicKey,
				paymentTreasury: paymentTreasury.publicKey,
				toAccount: toAccount,
				presalePda: presalePDA,
				authority: user.publicKey,
				tokenProgram: splToken.TOKEN_PROGRAM_ID,
			},
			signers: [user]
		});

		let toBalanceAfter = await getTokenAccountBalance(toAccount);
		let paymentBalanceAfter = await getTokenAccountBalance(paymentTreasury.publicKey);
		assert.equal(toBalanceAfter.sub(toBalanceBefore).toString(), paymentBalanceBefore.toString());
		assert.equal(paymentBalanceAfter, '0');
	}

	const unlockFractions = async (toAccount, user=payer) => {
		

		let [userVestingPDA, userVestingPDABump] = await anchor.web3.PublicKey.findProgramAddress(
			[Buffer.from("vesting"), user.publicKey.toBuffer(), presaleAccount.publicKey.toBuffer(), program.programId.toBuffer()], 
			program.programId
		);
		let vestingInfo = await program.account.vestingInfo.fetch(userVestingPDA);
		
		let toBalanceBefore = await getTokenAccountBalance(toAccount);
		let vestingBalanceBefore = await getTokenAccountBalance(vestingInfo.vestingAccount);
		console.log(vestingBalanceBefore.toString());
		
		await program.rpc.unlockFractions(userVestingPDABump, {
			accounts: {
                presaleAccount: presaleAccount.publicKey,
				fractionTreasury: fractionTreasury.publicKey,
				paymentTreasury: paymentTreasury.publicKey,
				toAccount: toAccount,
				vestingAccount: vestingAccount.publicKey,
				userVestingPda: userVestingPDA,
				signer: user.publicKey,
				tokenProgram: splToken.TOKEN_PROGRAM_ID
			},
			signers: [user]
		});

		let toBalanceAfter = await getTokenAccountBalance(toAccount);
		let vestingBalanceAfter = await getTokenAccountBalance(vestingInfo.vestingAccount);
		assert.equal(toBalanceAfter.sub(toBalanceBefore).toString(), vestingBalanceBefore.toString());
		assert.equal(vestingBalanceAfter.toString(), '0');
	}

	const checkPresaleInfo = async (fractionsSold, started) => {
		fractionsSold = (fractionsSold * 10**DECIMALS).toString();
		let priceString = (price * 10**DECIMALS).toString();
		let maxAmountString = (maxAmount * 10**DECIMALS).toString();
		let presaleInfo = await program.account.presaleInfo.fetch(presaleAccount.publicKey);
		assert.equal(presaleInfo.fractionTreasury.toString(), fractionTreasury.publicKey.toString());
		assert.equal(presaleInfo.paymentTreasury.toString(), paymentTreasury.publicKey.toString());
		assert.equal(presaleInfo.accessTreasury.toString(), accessTreasury.publicKey.toString());
		assert.equal(presaleInfo.authority.toString(), payerKey.toString());
		assert.equal(presaleInfo.fractionsSold.toString(), fractionsSold);
		assert.equal(presaleInfo.fractionMint.toString(), fractionMint.publicKey.toString());
		assert.equal(presaleInfo.paymentMint.toString(), paymentMint.publicKey.toString());
		assert.equal(presaleInfo.accessMint.toString(), accessMint.publicKey.toString());
		assert.equal(presaleInfo.price.toString(), priceString);
		assert.equal(presaleInfo.maxAmount.toString(), maxAmountString);
		assert.closeTo(presaleInfo.presaleStart.toNumber(), startTimestamp, 2);
		assert.equal(presaleInfo.started, started);
		assert.equal(presaleInfo.presaleEnd, presaleEnd.toString());
		assert.equal(presaleInfo.vestingEnd, vestingEnd.toString());
	}

	const checkVestingInfo = async (vestingKey, user=payer) => {
		let [userVestingPDA, _] = await anchor.web3.PublicKey.findProgramAddress(
			[Buffer.from("vesting"), user.publicKey.toBuffer(), presaleAccount.publicKey.toBuffer(), program.programId.toBuffer()], 
			program.programId
		);
		let vestingInfo = await program.account.vestingInfo.fetch(userVestingPDA);
		assert.equal(vestingInfo.signer.toString(), user.publicKey.toString());
		assert.equal(vestingInfo.vestingAccount.toString(), vestingKey.toString());
	}

	before(async () => {
		[presalePDA, presalePDABump] = await anchor.web3.PublicKey.findProgramAddress(
			[Buffer.from("presale"), presaleAccount.publicKey.toBuffer(), program.programId.toBuffer()], 
			program.programId
		);
	})


    it('Initialise presale account', async () => {

		fractionMint = await createMint()
		paymentMint = { publicKey: splToken.NATIVE_MINT }
		accessMint = await createMint()

		payerFractionAccount = await fractionMint.createAccount(payerKey)
		await fractionMint.mintTo(payerFractionAccount, payerKey, [], totalFractions * 1e9);

        await program.rpc.initializePresale(
			presalePDABump,
			new BN(price * 1e9),
			new BN(maxAmount * 1e9),
			presaleEnd,
			vestingEnd, 
			{
				accounts: {
					presaleAccount: presaleAccount.publicKey,
					fractionTreasury: fractionTreasury.publicKey,
					paymentTreasury: paymentTreasury.publicKey,
					accessTreasury: accessTreasury.publicKey,
					presalePda: presalePDA,
					fractionMint: fractionMint.publicKey,
					paymentMint: paymentMint.publicKey,
					accessMint: accessMint.publicKey,
					authority: payer.publicKey,
					tokenProgram: splToken.TOKEN_PROGRAM_ID,
					rent: SYSVAR_RENT_PUBKEY,
					systemProgram: SystemProgram.programId,
				},
				signers: [presaleAccount, fractionTreasury, paymentTreasury, accessTreasury]
			}
			
		);

		await checkPresaleInfo(0, false);
    });

	it('Add fractions to presale', async () => {
		await addFractionsToPresale(totalFractions, payerFractionAccount);
		await checkPresaleInfo(0, false);
	});

	it('Only authority can add fractions', async () => {
		let badActor = Keypair.generate();
		let badFractionAccount = await fractionMint.createAccount(payerKey)
		await fractionMint.mintTo(badFractionAccount, payerKey, [], 1 * 1e9);
		try {
			await addFractionsToPresale(1, badFractionAccount, badActor);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "A has_one constraint was violated");
		}
	});

	it('Removes fractions', async () => {
		await removeFractionsFromPresale(totalFractions / 2, payerFractionAccount);
		await checkPresaleInfo(0, false);
	});

	it('Only authority can remove fractions', async () => {
		let badActor = Keypair.generate();
		let badFractionAccount = await fractionMint.createAccount(payerKey)
		try {
			await removeFractionsFromPresale(1_000, badFractionAccount, badActor);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "A has_one constraint was violated");
		}
	});

	it('Init vesting account', async () => {
		await initVestingAccount();
		await checkPresaleInfo(0, false);
		await checkVestingInfo(vestingAccount.publicKey);
	})

	it("Cannot unlock vesting fractions if vesting account is empty", async () => {
		let toAccount = await fractionMint.createAccount(payerKey);
		try {
			await unlockFractions(toAccount);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "There are no tokens vested in the account");
		}
	})

	it("Cannot buy when presale has not started", async () => {
		payerAccessAccount = await accessMint.createAccount(payerKey);
		await getAccessTokens(payerAccessAccount, 1);
		let purchaseAmount = 1_000;
		let paymentAmount = (purchaseAmount * price);
		let userPaymentAccount = await createNativeTokenAccount(paymentAmount);
		try {
			await purchaseFractions(payerAccessAccount, userPaymentAccount, purchaseAmount);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "The presale has not yet started");
		}
	});

	it('Only authority can start presale', async () => {
		let badActor = Keypair.generate();
		try {
			await program.rpc.startPresale({
				accounts: {
					presaleAccount: presaleAccount.publicKey,
					authority: badActor.publicKey
				},
				signers: [badActor]
			});
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "A has_one constraint was violated");
		}
	});

	it('Start presale', async () => {
		await program.rpc.startPresale({
			accounts: {
				presaleAccount: presaleAccount.publicKey,
				authority: payer.publicKey
			}
		});
		startTimestamp = getCurrentTimestamp();
		await checkPresaleInfo(0, true);
	});
	
	it('Cannot start presale more than once', async () => {
		try {
			await program.rpc.startPresale({
				accounts: {
					presaleAccount: presaleAccount.publicKey,
					authority: payer.publicKey
				}
			});
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "The presale cannot be started more than once");
		}
	});

	it('Purchase fractions', async () => {
		payerAccessAccount = await accessMint.createAccount(payerKey);
		await getAccessTokens(payerAccessAccount, 1);
		let purchaseAmount = 1_000;
		let paymentAmount = purchaseAmount * price;
		let userPaymentAccount = await createNativeTokenAccount(paymentAmount);
		await purchaseFractions(payerAccessAccount, userPaymentAccount, purchaseAmount);
		await checkPresaleInfo(1_000, true);
		await checkVestingInfo(vestingAccount.publicKey);
	});

	it("Cannot unlock vesting fractions if vesting period hasn't ended", async () => {
		let toAccount = await fractionMint.createAccount(payerKey);
		try {
			await unlockFractions(toAccount);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "The vesting period has not finished");
		}
	})

	it('Only authority can collect funds', async () => {
		let badActor = Keypair.generate();
		let userPaymentAccount = await createNativeTokenAccount(0, badActor);
		try {
			await collectPayments(userPaymentAccount, badActor);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "A has_one constraint was violated");
		}
	});

	it('Collect funds', async () => {
		let userPaymentAccount = await createNativeTokenAccount(0);
		await collectPayments(userPaymentAccount);
	});

	it("Cannot buy fractions without enough funds", async () => {
		payerAccessAccount = await accessMint.createAccount(payerKey);
		await getAccessTokens(payerAccessAccount, 1);
		let purchaseAmount = 1_000;
		let paymentAmount = (purchaseAmount * price) - 1;
		let userPaymentAccount = await createNativeTokenAccount(paymentAmount);
		try {
			await purchaseFractions(payerAccessAccount, userPaymentAccount, purchaseAmount);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "The user doesn't have enough funds in their account to make the purchase");
		}
	});

	it("Cannot buy fractions without an access token", async () => {
		payerAccessAccount = await accessMint.createAccount(payerKey);
		let purchaseAmount = 1_000;
		let paymentAmount = (purchaseAmount * price);
		let userPaymentAccount = await createNativeTokenAccount(paymentAmount);
		try {
			await purchaseFractions(payerAccessAccount, userPaymentAccount, purchaseAmount);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "The user is missing a valid access token");
		}
	});

	it("Cannot buy 0 fractions", async () => {
		payerAccessAccount = await accessMint.createAccount(payerKey);
		await getAccessTokens(payerAccessAccount, 1);
		let purchaseAmount = 0;
		let paymentAmount = (purchaseAmount * price);
		let userPaymentAccount = await createNativeTokenAccount(paymentAmount);
		try {
			await purchaseFractions(payerAccessAccount, userPaymentAccount, purchaseAmount);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "Amount must be greater than zero");
		}
	});
	
	it("Cannot buy more than max fractions", async () => {
		payerAccessAccount = await accessMint.createAccount(payerKey);
		await getAccessTokens(payerAccessAccount, 1);
		let purchaseAmount = 1_001;
		let paymentAmount = (purchaseAmount * price);
		let userPaymentAccount = await createNativeTokenAccount(paymentAmount);
		try {
			await purchaseFractions(payerAccessAccount, userPaymentAccount, purchaseAmount);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "The user requested to purchase an amount greater than the maximum allowed");
		}
	});

	it('Cannot remove more fractions than there are in the fraction treasury', async () => {
		try {
			await removeFractionsFromPresale(500_000, payerFractionAccount);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "There are not enough tokens in the fraction treasury to satisfy the request");
		}
	});

	it('Removes fractions', async () => {
		await removeFractionsFromPresale(498_500, payerFractionAccount);
		await checkPresaleInfo(1_000, true);
	});

	it("Cannot buy more than amount in fraction treasury", async () => {
		payerAccessAccount = await accessMint.createAccount(payerKey);
		await getAccessTokens(payerAccessAccount, 1);
		let purchaseAmount = 1_000;
		let paymentAmount = (purchaseAmount * price);
		let userPaymentAccount = await createNativeTokenAccount(paymentAmount);
		try {
			await purchaseFractions(payerAccessAccount, userPaymentAccount, purchaseAmount);
			assert.ok(false);
		} catch (err) {
			assert.equal(err.toString(), "There are not enough tokens in the fraction treasury to satisfy the request");
		}
		purchaseAmount = 500;
		await purchaseFractions(payerAccessAccount, userPaymentAccount, purchaseAmount);
		await checkPresaleInfo(1_500, true);
	});

});
