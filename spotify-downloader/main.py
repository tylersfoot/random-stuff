import clspotify

# Set your Spotify API credentials
clspotify.set_credentials(client_id='your_client_id', client_secret='your_client_secret')

# Search for a track
track_results = clspotify.search('Never Gonna Give You Up', type='track', limit=1)

# Get the first track from the search results
track = track_results['tracks']['items'][0]

# Print the track details
print(f"Track name: {track['name']}")
print(f"Artist: {track['artists'][0]['name']}")
print(f"Album: {track['album']['name']}")
