package org.aeriagloris.rss.model

import org.simpleframework.xml.Element
import org.simpleframework.xml.ElementList
import org.simpleframework.xml.Root

@Root(name = "channel", strict = false)
class FeedChannel(
    @field:Element(name = "title")
    var title: String? = null,

    @field:Element(name = "description")
    var description: String? = null,

    @field:Element(name = "link")
    var link: String? = null,

    @field:Element(name = "language")
    var language: String? = null,

    @field:Element(name = "copyright")
    var copyright: String? = null,

    @field:Element(name = "ttl")
    var ttl: String? = null,

    @field:ElementList(inline = true, name = "item")
    var feedItems: List<AlertItem>? = null
)
