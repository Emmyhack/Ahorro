import './App.css'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { useWallet } from '@solana/wallet-adapter-react'

function App() {
	const { publicKey } = useWallet()

	return (
		<div style={{ display: 'flex', flexDirection: 'column', gap: 16, padding: 24 }}>
			<div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
				<h2>Ahorro — Thrift & Savings (Devnet)</h2>
				<WalletMultiButton />
			</div>

			{publicKey ? (
				<div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
					<div style={{ border: '1px solid #eee', borderRadius: 8, padding: 16 }}>
						<h3>Create Group</h3>
						<p>Initialize a thrift group with USDC and insurance percentage.</p>
						<button disabled>Coming soon</button>
					</div>
					<div style={{ border: '1px solid #eee', borderRadius: 8, padding: 16 }}>
						<h3>My Groups</h3>
						<p>Track contributions, current cycle position, insurance pool.</p>
						<button disabled>Coming soon</button>
					</div>
				</div>
			) : (
				<p>Connect a wallet to get started.</p>
			)}
		</div>
	)
}

export default App
