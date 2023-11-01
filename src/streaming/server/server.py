import socket, os, time
import asyncio
from websockets.server import serve
import threading

global curr_socket
global locald
locald = 0
global x

async def main_func(websocket):
    async for message in websocket:
        print(message)
        global curr_socket
        curr_socket = websocket
        global locald
        locald = message;

async def do_send(remote):
    await curr_socket.send(remote[0:len(remote)-1])
    time.sleep(5)

async def main():
    async with serve(main_func, "0.0.0.0", 8765):
        await asyncio.Future()  # run forever

def thread_function():
     asyncio.run(main())


x = threading.Thread(target=thread_function, args=())
x.start()


HOST = '0.0.0.0'
PORT = 8787

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.bind((HOST, PORT))
    PORT = PORT + 1
    print("listening to", PORT-1);
    s.listen()
    while 1:
      conn, addr = s.accept()
      remote= b''
      with conn:
          while True:
              if conn == 0:
                  break;
              data = conn.recv(1024)
              if data == b'Hello!':
                  while locald == 0:
                      time.sleep(5)

                  sdp = locald
                  print(sdp)
                  sdp = sdp + "\n";
                  i = 0
                  while i != len(sdp):
                      n = conn.send(bytes(sdp[i:], 'utf-8'))
                      i = i + n
                  conn.send(b'');
                  conn.close();
                  break;
                  locald = 0
              elif len(data) != 0:
                  remote = remote + data
                  if remote[len(remote)-1] == 10:
                      loop = asyncio.get_event_loop()
                      loop.run_until_complete(do_send(remote))
                      conn.close()
                      conn = 0
