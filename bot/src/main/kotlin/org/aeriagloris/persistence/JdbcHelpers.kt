package org.aeriagloris.persistence

import java.sql.*

fun PreparedStatement.setValue(index: Int, value: Any?) {
    when(value) {
        is String -> setString(index + 1, value)
        is Int -> setInt(index + 1, value)
        is Date -> setDate(index + 1, value)
    }
}

fun PreparedStatement.setValues(values: Collection<Any?>) {
    values.forEachIndexed { index, value -> setValue(index, value) }
}

fun Connection.executeInsert(table: String, vararg columns: Pair<String, Any?>): Int {
    val sql = buildInsertSql(table, columns.map { it.first })
    val stmt = prepareStatement(sql, Statement.RETURN_GENERATED_KEYS)
    stmt.setValues(columns.map { it.second })
    try {
        stmt.executeUpdate()
        val rs = stmt.generatedKeys
        if (rs.next()) {
            return rs.getInt(1)
        }
        return 0
    }
    finally {
        stmt.close()
    }
}

private fun buildInsertSql(table: String, columns: Collection<String>): String {
    val builder = StringBuilder("insert into $table(")
    builder.append(columns.joinToString(","))
    builder.append(") values(").append("?,".repeat(columns.size).trimEnd(',')).append(")")
    return builder.toString()
}

fun <T : Any> Connection.executeQuery(query: String, param: Any, callback: (ResultSet) -> T): T? {
    val stmt = prepareStatement(query)
    stmt.setValue(0, param)
    try {
        val rs = stmt.executeQuery()
        if (!rs.next()) {
            return null
        }
        return callback(rs)
    }
    finally {
        stmt.close()
    }
}

fun <T : Any> Connection.executeListQuery(query: String, param: Any, callback: (ResultSet) -> T): List<T> {
    val stmt = prepareStatement(query)
    stmt.setValue(0, param)
    try {
        val rs = stmt.executeQuery()
        val results = arrayListOf<T>()
        while (rs.next()) {
            results.add(callback(rs))
        }
        return results
    }
    finally {
        stmt.close()
    }
}

fun Connection.executeUpdate(query: String, vararg params: Any?) {
    val stmt = prepareStatement(query)
    stmt.setValues(params.toList())
    stmt.executeUpdate()
}
