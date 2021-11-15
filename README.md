<p align="center">
  <img
    src="images/axyn.png"
    alt="Axyn logo"
  />
</p>

# Axyn for Matrix

## Environment Variables

### `AXYN_MATRIX_STORE_PATH`

The directory that should be used for the database and Matrix state storage.

### `AXYN_MATRIX_HOMESERVER`

The URL of the homeserver which we want to connect to.

### `AXYN_MATRIX_USER_ID`

The user which will be used when we log in to the homeserver.

### `AXYN_MATRIX_USER_PASSWORD`

Password for the user given in `AXYN_MATRIX_USER_ID`.

### `AXYN_MATRIX_DEVICE_ID`

A unique identifier that distinguishes this client instance.

### `AXYN_MATRIX_DEVICE_NAME`

The name to give to a new device. If the device ID already exists, its name
will not be changed.
