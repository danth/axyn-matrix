{ fetchurl, jq, runCommand }:

let
  train = fetchurl {
    name = "craigslist-train";
    url = "https://worksheets.codalab.org/rest/bundles/0xda2bae7241044dbaa4e8ebb02c280d8f/contents/blob/";
    sha256 = "Ucz49q6cmJkCRKLo1JJ+eDyfCd+Amh27mVJ3wV/dyqA=";
  };

  dev = fetchurl {
    name = "craigslist-dev";
    url = "https://worksheets.codalab.org/rest/bundles/0xb0fe71ca124e43f6a783324734918d2c/contents/blob/";
    sha256 = "eFmx165qkrM0r79F5tQmolO9v6kHuBRXzsms2A0ZCv0=";
  };

  test = fetchurl {
    name = "craigslist-test";
    url = "https://worksheets.codalab.org/rest/bundles/0x54d325bbcfb2463583995725ed8ca42b/contents/blob/";
    sha256 = "yALxX4DqMGbUKTdTkzGdcjTarL1qJqatWv0K14ovdzY=";
  };

in {
  name = "craigslist-bargains";
  buildScript = ./build.py;

  # The data is split into train, dev and test.
  # As Axyn only uses data for training, we combine all three.
  input = runCommand "craigslist-bargains" { } ''
    ${jq}/bin/jq -n '[inputs] | add' ${train} ${dev} ${test} > $out
  '';
}
