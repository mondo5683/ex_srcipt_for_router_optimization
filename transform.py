import json
import csv

# === Step 1: CSVファイルを辞書に読み込む ===
csv_file = "./device.csv"  # CSVファイルの名前
id_to_value = {}

with open(csv_file, newline='', encoding='utf-8') as f:
    reader = csv.reader(f)
    for row in reader:
        if row:  # 空行を無視
            id_ = int(row[0].strip())
            value = float(row[1].strip())
            id_to_value[id_] = value

# === Step 2: JSONファイルを読み込む ===
json_file = "./topology.json"  # JSONファイルの名前
with open(json_file, 'r', encoding='utf-8') as f:
    data = json.load(f)

# === Step 3: JSONデータを変換する ===
mapped_data = {}
for key, ids in data.items():
    mapped_values = [id_to_value.get(id_, None) for id_ in ids]
    mapped_data[key] = mapped_values

# === Step 4: 結果を新しいJSONファイルに書き出す ===
output_file = "mapped_data.json"
with open(output_file, 'w', encoding='utf-8') as f:
    json.dump(mapped_data, f, indent=4)

print(f"変換後のJSONデータを {output_file} に保存しました！")

