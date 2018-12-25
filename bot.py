import discord
import youtube_dl
from discord.ext import commands

# Creates the bot and specifies the prefix
client = commands.Bot(command_prefix="!")

@client.event
async def on_ready():
    print("online")

@client.command(pass_context=True)
async def join(ctx):
    channel = ctx.message.author.voice.voice_channel
    await client.join_voice_channel(channel)

@client.command(pass_context=True)
async def play(ctx):
    server = ctx.message.server
    voice_client = client.voice_client_in(server)
    player = await voice_client.create_ytdl_player("http://icecast.omroep.nl/radio2-bb-mp3")
    player.start()

# actually run the bot
client.run("NTI3MTE1MTk1MDU5OTI5MTE4.DwPPPA.9yeoo_IGMzMVAwBLfATt6LdVCaw") # Needs to be moved to a file
