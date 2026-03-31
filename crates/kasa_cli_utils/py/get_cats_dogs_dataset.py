import os
from datasets import load_dataset
from PIL import Image
import io

# Load the dataset
ds = load_dataset("microsoft/cats_vs_dogs")

# Specify the output directory
output_dir = "cats_dogs_images"

# Create the output directory if it doesn't exist
os.makedirs(output_dir, exist_ok=True)


# Function to save an image
def save_image(example, idx):
    image = example["image"]
    label = "cat" if example["labels"] == 0 else "dog"
    filename = f"{label}_{idx}.jpg"
    image.save(os.path.join(output_dir, filename))


# Iterate through the dataset and save images
for split in ds.keys():
    print(f"Processing {split} split...")
    for idx, example in enumerate(ds[split]):
        save_image(example, idx)
        if idx % 100 == 0:
            print(f"Processed {idx} images")

print("Done!")
