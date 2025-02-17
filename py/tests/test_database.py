from sahomedb.prelude import Record, Collection, Config, Database


NAME = "vectors"  # Initial collection name.
DIMENSION = 128
LEN = 100


def create_test_database(path: str) -> Database:
    """Creates a new test database with an initial collection."""

    db = Database.new(path)
    assert db.is_empty()

    # Create a test collection with random records.
    records = Record.many_random(dimension=DIMENSION, len=LEN)
    config = Config.create_default()
    collection = Collection.from_records(config, records)

    # Save the collection to the database.
    db.save_collection(name=NAME, collection=collection)
    assert not db.is_empty()

    return db


def test_open():
    db = Database(path="data/101")
    assert db.is_empty()


def test_new():
    db = create_test_database(path="data/102")
    assert not db.is_empty()
    assert db.len() == 1


def test_get_collection():
    db = create_test_database(path="data/103")
    collection = db.get_collection(name=NAME)
    assert collection.len() == LEN


def test_save_collection():
    db = create_test_database(path="data/104")

    # Create a new collection and save it to the database.
    config = Config.create_default()
    collection = Collection(config=config)
    db.save_collection(name="test", collection=collection)

    assert db.len() == 2


def test_delete_collection():
    db = create_test_database(path="data/105")
    db.delete_collection(name=NAME)
    assert db.is_empty()