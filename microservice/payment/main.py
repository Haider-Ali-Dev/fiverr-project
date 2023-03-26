from fastapi.middleware.cors import CORSMiddleware
import stripe
import os
import uuid
from fastapi import FastAPI
from pydantic import BaseModel
import psycopg2
app = FastAPI()
API_KEY = os.environ.get('API_KEY')
stripe.api_key = API_KEY

origins = [
    'http://localhost:3000',
    'http://localhost:4200'
]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

conn = psycopg2.connect(
    dbname="ichiban_payment",
    user="postgres",
    password="haider",
    host="localhost",
    port="5432"
)


class Payment(BaseModel):
    id: str
    email: str
    card_number: str
    expiry_date: str
    cvc: str
    tty_of_points: str


@app.post('/pay')
async def payment(data: Payment):
    amount = 0
    if data.tty_of_points == 'BRONZE':
        amount = 25
    elif data.tty_of_points == 'SILVER':
        amount = 50
    elif data.tty_of_points == 'GOLD':
        amount = 100

    token = stripe.Token.create(
        card={
            "number": data.card_number,
            "exp_month": data.expiry_date.split('/')[0],
            "exp_year": data.expiry_date.split('/')[1],
            "cvc": data.cvc,
            "name": data.email
        }
    )

    charge = stripe.Charge.create(
        amount=amount,
        currency="usd",
        source=token.id,
        description="Charge for "+data.email,
        receipt_email=data.email,

    )

    if charge.status == 'succeeded':
        cur = conn.cursor()
        cur.execute("INSERT INTO payment (id, email, card_number, expiry_date, cvc, tty_of_points, amount) VALUES (%s, %s, %s, %s, %s, %s, %s)",
                    (data.id, data.email, data.card_number, data.expiry_date, data.cvc, data.tty_of_points, amount))
        conn.commit()
        cur.close()
        return {'status': 'success'}
    else:
        return {'status': 'failed'}
