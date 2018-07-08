import hashlib
import json
import requests

from flask import Flask, jsonify, request
from time import time
from urllib.parse import urlparse
from uuid import uuid4


class Blockchain:
    def __init__(self):
        self.chain = []
        self.current_transactions = []
        self.nodes = set()

        self.new_block(proof=100, previous_hash='1')

    def register_node(self, address: str):
        self.nodes.add(urlparse(address).netloc)

    def valid_chain(self, chain) -> bool:
        last_block = chain[0]
        current_index = 1

        while current_index < len(chain):
            block = chain[current_index]

            if block['previous_hash'] != self.hash(last_block):
                return False
            if not self.valid_proof(last_block['proof'], block['proof']):
                return False

            last_block = block
            current_index += 1

        return True

    def resolve_conflicts(self):
        neighbours = self.nodes
        chain = None
        chain_len = len(self.chain)

        for node in neighbours:
            response = requests.get(f'http://{node}/chain')

            if response.status_code == 200:
                other_chain = response.json()['chain']
                other_chain_len = response.json()['length']

                if chain_len < other_chain_len and self.valid_chain(other_chain):
                    chain = other_chain
                    chain_len = other_chain_len

        if chain:
            self.chain = chain

            return True
        return False

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
            'amount': float(amount)
        })

        return self.last_block['index'] + 1

    @staticmethod
    def hash(block):
        return hashlib.sha256(
            json.dumps(block, separators=(',', ':'), sort_keys=True).encode()
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
node_identifier = str(uuid4()).replace('-', '')


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
    proof = blockchain.proof_of_work(blockchain.last_block['proof'])

    blockchain.new_transaction(sender="0", recipient=node_identifier, amount=1)

    block = blockchain.new_block(proof, None)

    return jsonify({
        'message': 'New Block forged',
        'index': block['index'],
        'transactions': block['transactions'],
        'proof': block['proof'],
        'previous_hash': block['previous_hash']
    }), 200


@app.route('/chain', methods=['GET'])
def full_chain():
    return jsonify({
        'chain': blockchain.chain,
        'length': len(blockchain.chain)
    }), 200


@app.route('/nodes/register', methods=['POST'])
def register_nodes():
    values = request.get_json()

    if values is None:
        return 'Error: please supply a valid list of nodes', 400

    nodes = values.get('nodes')

    if nodes is None:
        return 'Error: please supply a valid list of nodes', 400

    for node in nodes:
        blockchain.register_node(node)

    return jsonify({
        'message': 'Nodes have been added',
        'total_nodes': list(blockchain.nodes)
    }), 201


@app.route('/nodes/resolve', methods=['GET'])
def consensus():
    if blockchain.resolve_conflicts():
        response = {
            'message': 'Our chain was replaced',
            'new_chain': blockchain.chain
        }
    else:
        response = {
            'message': 'Our chain is authoritative',
            'chain': blockchain.chain
        }

    return jsonify(response), 200


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5001)
