
Problem statement:

Scraping catalog and product data pages daily will create a big pressure on amount of data storage. While in general, that data has not much value after few days, we should still keep it and be able to reuse it later if needed.

Daily scrape of 1mil pages, with their avg size being 200k, we are talking about 200GB daily , 1.4TB weekly and 6TB monthly of ingress data.

I tryed compressing html files and I got 240k original to take 28kb zipped while 23kb when compressed with xz.

That would reduce storage needs to 20GB daily,140GB weekly and 600 GB monthly


Minio is a dead project, since few months ago. there are forks, check them out

seeweedfs is an alternative worth looking at
