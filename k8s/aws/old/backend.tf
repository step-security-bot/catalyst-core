terraform {
  backend "s3" {
    bucket = "catalyst-dev-tfstate"
    key    = "tfstate"
    region = "eu-west-2"
  }
}
 