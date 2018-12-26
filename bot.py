import asyncio
from datetime import datetime
import discord
import api
from discord.ext import commands
import os

# Creates the bot and specifies the prefix
bot = commands.Bot(command_prefix="top2000-")
players = {}
channels = []
current_song = 0
song_delay = 15


@bot.event
async def on_ready():
    for channel in bot.get_all_channels():
        if channel.name == "TOP2000":
            await bot.join_voice_channel(channel)
    print("Started...")


@bot.command(pass_context=True)
async def play(ctx):
    server = ctx.message.server
    voice_client = bot.voice_client_in(server)
    player = await voice_client.create_ytdl_player("http://icecast.omroep.nl/radio2-bb-mp3")
    player.start()
    players[server.id] = player
    channels.append(ctx.message.channel)
    print("Playing...")
    await bot.say(embed=generate_current_song_embed())


@bot.command(pass_context=True)
async def pause(ctx):
    server = ctx.message.server
    players[server.id].stop()


def generate_current_song_embed():
    on_air = api.get_now_on_air()["songfile"]
    on_air_details = api.get_now_on_air_details(on_air)
    on_air_full_list = api.get_now_on_air_from_full_list(on_air)

    embed = discord.Embed(title=on_air["title"] + " - " + on_air["artist"])

    embed.add_field(name="Description", value=on_air_details["description"])
    embed.url = "https://www.nporadio2.nl" + on_air_full_list["url"]

    try:
        img = api.get_img_url(on_air['songversion']['image'][0]['url'])
    except KeyError:
        img = "https://i.imgur.com/Z3yujMQ.png"

    embed.set_thumbnail(url=img)

    footer = str(on_air_full_list["pos"])
    if on_air_full_list["prv"] != 0:
        footer += " (last year: " + str(on_air_full_list["prv"]) + ")"
    embed.add_field(name="Position", value=footer)

    return embed


@bot.command()
async def song():
    await bot.say(embed=generate_current_song_embed())


async def check_if_new():
    await bot.wait_until_ready()

    song_id = api.get_now_on_air()['id']

    global current_song

    print("checking " + str(song_id) + " vs " + str(current_song))  # Debug message

    if not song_id == current_song:  # Still compare because as sometimes the DJ does not switch immediately
        if current_song == 34096:  # Bohemian
            await happy_new_year()

        current_song = song_id

        for c in channels:
            if players[c.server.id].is_playing():
                if song_id == 34096: # BOHEMIANNNN
                    await bot.send_message(c, content="LAST SONG!")
                await bot.send_message(c, embed=generate_current_song_embed())


async def happy_new_year():

    # We need to add a wait until it is over here

    for p in players:
        players[p].stop()

    for c in channels:
        await bot.send_message(c, content="Happy new year!!! Until next year :)")

    os._exit(0)


async def background():
    await bot.wait_until_ready()
    while not bot.is_closed:
        await check_if_new()

        nowonair = api.get_now_on_air()
        global current_song
        if nowonair['id'] != current_song:
            await asyncio.sleep(song_delay)
        else:
            # TZ Offset hardcoded because because can't get python to handle it properly
            end = datetime.strptime(nowonair['stopdatetime'], "%Y-%m-%dT%H:%M:%S+01:00")
            run_at = end - datetime.now()
            delay = max(int(run_at.total_seconds() + song_delay), song_delay)
            await asyncio.sleep(delay)  # And finally wait for the calculated delay

if __name__ == "__main__":
    bot.loop.create_task(background())
    bot.run("NTI3MTE1MTk1MDU5OTI5MTE4.DwPPPA.9yeoo_IGMzMVAwBLfATt6LdVCaw")  # Needs to be moved to a file

