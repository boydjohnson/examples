
import argparse
import asyncio
import json
import logging
from random import randint
import sys

from aiohttp import web

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(logging.DEBUG)
parser = argparse.ArgumentParser("server")

parser.add_argument(
    '--host',
    default='localhost',
    type=str,
    help="The scheme and host to bind the server to.")

parser.add_argument(
    "--port",
    default=8080,
    type=int,
    help="The port to bind the server to.")


async def handler(request):
    json_input = await request.json()
    val = int(json_input['input'])
    LOGGER.warning("Value %s", val)
    wait = randint(1, 5)
    amt = randint(1, 5)
    asyncio.sleep(wait)
    LOGGER.warning("Sending back")
    return web.Response(text=json.dumps({'output': val + amt}))



def main(args=sys.argv[1:]):
    opts = parser.parse_args(args=args)

    app = web.Application()
    app.router.add_post('/', handler)

    web.run_app(app, host=opts.host, port=opts.port)
