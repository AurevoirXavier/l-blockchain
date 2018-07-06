import hashlib
import json

from time import time
from urllib.parse import urlparse

from flask import Flask, jsonify, request


class Blockchain:
    def __init__(self):
        self.chain = []
        self.current_transactions = []
        self.nodes = set()

        self.new_block(proof=100, previous_hash=1)

    def new_block(self, proof, previous_hash=None):
        block = {
            'index': len(self.chain) + 1,
            'timestamp': time(),
            'transactions': self.current_transactions,
            'proof': proof,
            'previous_hash': previous_hash or self.hash(self.last_block)
        }

        self.current_transactions = []
        self.chain.append(block)

        return block

    def new_transaction(self, sender, recipient, amount) -> int:
        self.current_transactions.append({
            'sender': sender,
            'recipient': recipient,
            'amount': amount
        })

        return self.last_block['index'] + 1

    @staticmethod
    def hash(block):
        return hashlib.sha256(
            json.dumps(block, sort_keys=True).encode()
        ).hexdigest()

    @property
    def last_block(self):
        return self.chain[-1]

    def proof_of_work(self, last_proof: int) -> int:
        proof = 0

        while self.valid_proof(last_proof, proof) is False:
            proof += 1

        return proof

    def valid_proof(self, last_proof, proof: int) -> bool:
        return hashlib.sha256(
            f'{last_proof}{proof}'.encode()
        ).hexdigest()[0:4] == '0000'


app = Flask(__name__)
blockchain = Blockchain()


@app.route('/transactions/new', methods=['POST'])
def new_transaction():
    values = request.get_json()
    required = ['sender', 'recipient', 'amount']

    if values is None:
        return 'Missing values', 400

    if not all(k in values for k in required):
        return 'Missing values', 400

    index = blockchain.new_transaction(
        values['sender'],
        values['recipient'],
        values['amount']
    )

    return jsonify({
        'message': f'Transaction will be added to Block {index}'
    }), 201


@app.route('/mine', methods=['GET'])
def mine():
    return 'We\'ll mine a new block'


@app.route('/chain', methods=['GET'])
def full_chain():
    return jsonify({
        'chain': blockchain.chain,
        'length': len(blockchain.chain)
    }), 200


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
