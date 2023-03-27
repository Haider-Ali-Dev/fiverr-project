from fastapi.middleware.cors import CORSMiddleware
import stripe
import os
from datetime import datetime, timezone
import uuid
from fastapi import FastAPI
from pydantic import BaseModel
import psycopg2
app = FastAPI()
API_KEY = 'sk_test_51Mpc0PCf32Q6vHEwGbQDloAFfEfAxqHXZk9MtuP1VZtXKxxOqh06E8MOFFeG6glDDDvXW6MqKUH0OfnkvnLxRQHd00pVgMZsSl'
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
    name: str
    email: str
    card_number: str
    expiry_date: str
    cvc: str
    tty_of_points: str


@app.post('/pay')
async def payment(data: Payment):
    amount = 0
    if data.tty_of_points == 'BRONZE':
        amount = 2500
    elif data.tty_of_points == 'SILVER':
        amount = 5000
    elif data.tty_of_points == 'GOLD':
        amount = 100

    data.email = 'ebusiness413@gmail.com'
    data.id = str(uuid.uuid4())
    token = stripe.Token.create(
        card={
            "number": data.card_number,
            "exp_month": data.expiry_date.split('/')[0],
            "exp_year": data.expiry_date.split('/')[1],
            "cvc": data.cvc,
            "name": data.name
        
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
        cur.execute("INSERT INTO payments (id, user_email, user_id, created_at, amount, tty_points) VALUES(%s, %s, %s, %s, %s, %s)"
                    , (str(uuid.uuid4()), data.email, data.id, datetime.now(timezone.utc), amount, data.tty_of_points))
        conn.commit()
        cur.close()
        return {'status': 'success'}
    else:
        return {'status': 'failed'}
