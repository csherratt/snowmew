# Architecture #

Snowmew is broken up into a number of smaller parts. This is largely to keep the modules loosely couples, but also to help improve compile speeds ect.

## Core ##

Core is mostly a data management layer. It contains the database that snowmew is build around. It also defines the interface for managers, which is how most objects inside of snowmew are linked.

### Database ###

All of the data inside of snowmew is stored in a large copy-on-write database. The database is generational, meaning that each 'frame' the database is duplicated and can be held onto. This is useful since each manager is always working on a stable copy of the database. They can also be working on data from different generations. The Render / Audio manager will tend to be working on the oldest generation, They can render out the older data while newer data is being prepared upstream.

### Managers ###

A manager is an asynchronous component that works on it's own piece of the engine. The `main` of the engine is mostly responsible for handing each manager a copy of the current database.

There are two types of managers, passive managers and active managers. Passive managers are sinks for data, where active managers write out changes to the database.


## Render ##

## Audio ##

## Physics ##

## AI ##

## Network ##

## Input ##