#!/bin/bash
sudo cp ../Rocket.toml /home/passi/.weatherapp/Rocket.toml
sudo cp weather-webapp.service /etc/systemd/system/weather-webapp.service
sudo cp weather-ingest.service /etc/systemd/system/weather-ingest.service

sudo systemctl daemon-reload

sudo systemctl enable weather-ingest.service
sudo systemctl start weather-ingest.service

sudo systemctl enable weather-webapp.service
sudo systemctl start weather-webapp.service