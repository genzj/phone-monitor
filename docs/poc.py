import hmac
from base64 import b64encode
from datetime import datetime, timezone
from typing import Optional
from urllib.parse import quote_plus

import boto3
from requests import Session, session


class SMSForwarderAPI:
    session: Session
    base_url: str
    secret: str

    def __init__(
        self,
        base_url: str,
        secret: str,
    ) -> None:
        base_url = base_url.rstrip('/')
        self.base_url = base_url
        self.secret = secret
        self.session = session()

    def _sign(self, timestamp: int) -> str:
        sign = hmac.digest(
            self.secret.encode('utf-8'),
            f'{timestamp}\n{self.secret}'.encode('utf-8'),
            'sha256',
        )
        return quote_plus(b64encode(sign))

    def _make_body(self, data: dict) -> dict:
        ts = round(datetime.now(timezone.utc).timestamp() * 1000)
        return {
            "data": data,
            "timestamp": ts,
            "sign": self._sign(ts),
        }

    def _make_headers(self) -> dict:
        return {
            'Content-Type': 'application/json; charset=utf-8',
        }

    def _invoke(
        self,
        path: str,
        data: Optional[dict] = None,
        verify: bool = True,
    ) -> dict:
        if not data:
            data = dict()
        path = path.lstrip('/')
        with self.session.post(
            url=f'{self.base_url}/{path}',
            json=self._make_body(data),
            headers=self._make_headers(),
        ) as req:
            resp = req.json()
        if not verify:
            return resp

        try:
            sign_local = self._sign(resp['timestamp'])
            if sign_local != resp['sign']:
                raise ValueError(f'{sign_local} != {resp["sign"]}')
        except:
            raise RuntimeError('response from server is unverifiable')
        return resp

    def config(self) -> dict:
        return self._invoke('/config/query')

    def battery(self) -> dict:
        return self._invoke('/battery/query')


def report(phone_id: str, battery_level: float, timestamp: int):
    client = boto3.client('cloudwatch')
    client.put_metric_data(
        Namespace='phone',
        MetricData=[
            {
                'MetricName': 'battery',
                'Dimensions': [
                    {'Name': 'phone_id', 'Value': phone_id},
                ],
                'Timestamp': datetime.fromtimestamp(
                    timestamp / 1000.0, timezone.utc
                ),
                'Value': battery_level,
                'Unit': 'Percent',
            },
        ],
    )


import pprint

api = SMSForwarderAPI('http://192.168.1.28:5000', 'VyWatNuqAp6GYDG')
phone_info = {
    'config': api.config(),
    'battery': api.battery(),
}
pprint.pprint(phone_info)
report(
    phone_id=phone_info['config']['data']['extra_device_mark'],
    battery_level=float(phone_info['battery']['data']['level'].rstrip('%')),
    timestamp=phone_info['battery']['timestamp'],
)
