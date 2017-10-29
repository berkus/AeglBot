package org.aeriagloris.rss.model

import org.simpleframework.xml.Element
import org.simpleframework.xml.Root
import org.simpleframework.xml.Namespace

@Root(name = "item", strict = false)
class AlertItem (
    @field:Element(name = "guid")
    var guid: String? = null,

    @field:Element(name = "title")
    var title: String? = null,

    @field:Element(name = "author")
    var type: String? = null,

    @field:Element(name = "pubDate")
    var startDate: String? = null,

    @field:Element(name = "expiry", required = false)
    @field:Namespace(prefix="wf", reference="http://warframe.com/rss/v1")
    var expiryDate: String? = null,

    @field:Element(name = "faction", required = false)
    @field:Namespace(prefix="wf", reference="http://warframe.com/rss/v1")
    var faction: String? = null
)
