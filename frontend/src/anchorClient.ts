import { AnchorProvider, BN, Idl, Program, setProvider } from '@coral-xyz/anchor'
import { Connection, PublicKey } from '@solana/web3.js'
import { WalletContextState } from '@solana/wallet-adapter-react'

// Minimal subset of IDL for types; in production import generated IDL JSON
export const AHORRO_IDL: Idl = {
	version: '0.1.0',
	name: 'ahorro',
	instructions: [
		{
			name: 'createGroup',
			accounts: [
				{ name: 'authority', isMut: true, isSigner: true },
				{ name: 'usdcMint', isMut: false, isSigner: false },
				{ name: 'thriftGroup', isMut: true, isSigner: false },
				{ name: 'groupVaultAuthority', isMut: false, isSigner: false },
				{ name: 'insuranceVaultAuthority', isMut: false, isSigner: false },
				{ name: 'groupVault', isMut: true, isSigner: false },
				{ name: 'insuranceVault', isMut: true, isSigner: false },
				{ name: 'systemProgram', isMut: false, isSigner: false },
				{ name: 'tokenProgram', isMut: false, isSigner: false },
				{ name: 'associatedTokenProgram', isMut: false, isSigner: false },
			],
			args: [
				{ name: 'modelType', type: 'u8' },
				{ name: 'insuranceBps', type: 'u16' },
				{ name: 'cycleOrder', type: { vec: 'publicKey' } },
				{ name: 'contributionAmount', type: 'u64' },
			],
		},
	],
} as any

export function getProgram(wallet: WalletContextState) {
	const rpc = import.meta.env.VITE_SOLANA_RPC_ENDPOINT || 'https://api.devnet.solana.com'
	const connection = new Connection(rpc, 'confirmed')
	// Wallet adapter implements only needed interface
	const provider = new AnchorProvider(connection, wallet as any, { commitment: 'confirmed' })
	setProvider(provider)
	const programId = new PublicKey(import.meta.env.VITE_AHORRO_PROGRAM_ID)
	return new Program(AHORRO_IDL, programId, provider)
}

export type { Program }
export { BN }