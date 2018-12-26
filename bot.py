import asyncio
from datetime import datetime, timedelta
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


def error():
    os._exit(1)


bot.on_error(error)


@bot.event
async def on_ready():
    for channel in bot.get_all_channels():
        if channel.name == "top2000" and channel.type == discord.enums.ChannelType.voice:
            await bot.wait_until_ready()
            await bot.join_voice_channel(channel)
            server = channel.server
            voice_client = bot.voice_client_in(server)
            player = await voice_client.create_ytdl_player("https://icecast.omroep.nl/radio2-bb-mp3")
            player.start()
            players[server.id] = player
            print("Joined voice channel...")
        elif channel.name == "top2000" and channel.type == discord.enums.ChannelType.text:
            channels.append(channel)
            await bot.send_message(channel, embed=await generate_current_song_embed())
            print("Joined text channel...")


async def generate_current_song_embed(on_air_full = api.get_now_on_air()):
    on_air = on_air_full["songfile"]
    on_air_details = api.get_now_on_air_details(on_air)
    on_air_full_list = api.get_now_on_air_from_full_list(on_air)

    title_artist = on_air["title"] + " - " + on_air["artist"]

    now = (datetime.utcnow() + timedelta(hours=1))
    print("Playing song: " + title_artist + " at " + str(now.time()))

    embed = discord.Embed(title=title_artist)

    embed.add_field(name="Description", value=on_air_details["description"])
    embed.url = "https://www.nporadio2.nl" + on_air_full_list["url"]

    try:
        img = api.get_img_url(on_air["songversion"]["image"][0]["url"])
    except (KeyError, TypeError):
        img = "https://i.imgur.com/Z3yujMQ.png"

    game = discord.Game()
    game.name = title_artist
    game.url = img
    game.type = 2
    await bot.change_presence(game=game, afk=False)

    embed.set_thumbnail(url=img)

    footer = str(on_air_full_list["pos"])
    if on_air_full_list["prv"] != 0:
        footer += " (last year: " + str(on_air_full_list["prv"]) + ")"
    embed.add_field(name="Position", value=footer)

    return embed


@bot.command()
async def song():
    await bot.say(embed=await generate_current_song_embed())


@bot.command()
async def restart():
    error()


async def check_if_new():
    await bot.wait_until_ready()

    song = api.get_now_on_air()
    song_id = song['id']

    global current_song

    print("Checking: " + str(song_id) + " vs " + str(current_song))  # Debug message

    if not song_id == current_song:  # Still compare because as sometimes the DJ does not switch immediately
        if current_song == 34096:  # Bohemian
            await happy_new_year()

        current_song = song_id

        for c in channels:
            if players[c.server.id].is_playing():
                if song_id == 34096:  # BOHEMIANNNN
                    await bot.send_message(c, content="LAST SONG!")
                await bot.send_message(c, embed=await generate_current_song_embed(song))


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
            print("New song, id = " + str(nowonair['id']))
            await asyncio.sleep(song_delay)
        else:
            # TZ Offset hardcoded because because can't get python to handle it properly
            end = datetime.strptime(nowonair['stopdatetime'], "%Y-%m-%dT%H:%M:%S+01:00")
            now = (datetime.utcnow() + timedelta(hours=1))
            print("Now: " + str(now.time()))
            print("End: " + str(end.time()))

            run_at = end - now
            delay = max(int(run_at.total_seconds() + song_delay), song_delay)
            await asyncio.sleep(delay)  # And finally wait for the calculated delay

if __name__ == "__main__":
    bot.loop.create_task(background())
    bot.run("NTI3MTE1MTk1MDU5OTI5MTE4.DwPPPA.9yeoo_IGMzMVAwBLfATt6LdVCaw")  # Needs to be moved to a file

