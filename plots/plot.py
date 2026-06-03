import json
import matplotlib.pyplot as plt
import pandas as pd
import numpy as np
import os

# Create the figures directory if it doesn't exist
os.makedirs('plots', exist_ok=True)

# 1. Load the data
with open('plots/data.json', 'r') as f:
    data = json.load(f)

# ==========================================
# PART 1: LINE CHARTS (Evolution over Cycles)
# ==========================================

# Flatten and process the JSON into a Pandas DataFrame
rows = []
for run in data['runs']:
    for v in run['varients']:
        # Calculate Metadata Percentage Usage
        meta_pct = (v['max-metadata-size-bytes'] / v['max-total-memory-usage-bytes']) * 100
        
        rows.append({
            'subject': v['subject-name'],
            'cycle': v['cycle'],
            'hit_rate': v['hit-rate'],
            'throughput': v['operations-per-second'],
            'metadata_pct': meta_pct
        })

df = pd.DataFrame(rows)

# Group by subject and cycle, taking the mean of the metrics to show the overall trend.
df_agg = df.groupby(['subject', 'cycle']).mean().reset_index()

# Setup consistent styles for each subject
subjects = df_agg['subject'].unique()
colors = plt.cm.tab10(np.linspace(0, 1, len(subjects)))
markers = ['o', 's', '^', 'D', 'v', 'p']

style_map = {}
for i, subj in enumerate(subjects):
    style_map[subj] = {
        'color': colors[i],
        'marker': markers[i % len(markers)]
    }

# Plot 1: Hit Rate vs Cycles
plt.figure(figsize=(6, 4))
for subj in subjects:
    subj_data = df_agg[df_agg['subject'] == subj].sort_values('cycle')
    plt.plot(subj_data['cycle'], subj_data['hit_rate'], 
             color=style_map[subj]['color'], marker=style_map[subj]['marker'], linewidth=2)

plt.xlabel('Cycles', fontsize=12)
plt.ylabel('Hit Rate', fontsize=12)
plt.title('Evolution of Hit Rate', fontsize=14)
plt.grid(True, linestyle='--', alpha=0.7)
plt.tight_layout()
plt.savefig('plots/hitrate-vs-cycles.png', dpi=300)
plt.close()

# Plot 2: Metadata % Usage vs Cycles
plt.figure(figsize=(6, 4))
for subj in subjects:
    subj_data = df_agg[df_agg['subject'] == subj].sort_values('cycle')
    plt.plot(subj_data['cycle'], subj_data['metadata_pct'], 
             color=style_map[subj]['color'], marker=style_map[subj]['marker'], linewidth=2)

plt.xlabel('Cycles', fontsize=12)
plt.ylabel('Metadata Usage (%)', fontsize=12)
plt.title('Evolution of Metadata Overhead', fontsize=14)
plt.grid(True, linestyle='--', alpha=0.7)
plt.tight_layout()
plt.savefig('plots/metadata-vs-cycles.png', dpi=300)
plt.close()

# Plot 3: Throughput vs Cycles
plt.figure(figsize=(6, 4))
for subj in subjects:
    subj_data = df_agg[df_agg['subject'] == subj].sort_values('cycle')
    plt.plot(subj_data['cycle'], subj_data['throughput'], 
             color=style_map[subj]['color'], marker=style_map[subj]['marker'], linewidth=2)

plt.xlabel('Cycles', fontsize=12)
plt.ylabel('Throughput (Ops/Sec)', fontsize=12)
plt.title('Evolution of Cache Throughput', fontsize=14)
plt.grid(True, linestyle='--', alpha=0.7)
plt.tight_layout()
plt.savefig('plots/throughput-vs-cycles.png', dpi=300)
plt.close()

# Plot 4: Standalone Legend
fig_leg = plt.figure(figsize=(10, 1.2))
ax_leg = fig_leg.add_subplot(111)
ax_leg.axis('off')

handles = []
labels = []
for subj in subjects:
    h, = ax_leg.plot([], [], color=style_map[subj]['color'], 
                     marker=style_map[subj]['marker'], linewidth=2, label=subj, markersize=8)
    handles.append(h)
    labels.append(subj)

ax_leg.legend(handles=handles, labels=labels, loc='center', ncol=len(subjects), 
              frameon=False, fontsize=12)

fig_leg.savefig('plots/legend.png', bbox_inches='tight', dpi=300)
plt.close()

print("Successfully generated 3 cycle evolution plots and 1 standalone legend in the 'plots/' directory.")

# ==========================================
# PART 2: GROUPED BAR CHARTS (Final Performance by Model)
# ==========================================

# Extract ONLY the final cycle for each run
final_variants = []
for run in data['runs']:
    model_impl = run['setup']['model-implement-cache']
    # Get the variant with the maximum cycle number in this run
    final_variant = max(run['varients'], key=lambda x: x['cycle'])
    
    final_variants.append({
        'subject': final_variant['subject-name'],
        'model': model_impl,
        'hit_rate': final_variant['hit-rate'],
        'throughput': final_variant['operations-per-second'],
        'metadata': final_variant['max-metadata-size-bytes']
    })

df_final = pd.DataFrame(final_variants)

# Setup plotting function for grouped bars
def plot_grouped_bar(metric_col, title, ylabel, filename):
    plt.figure(figsize=(8, 5))
    
    subjects_final = df_final['subject'].unique()
    models = df_final['model'].unique()
    
    x = np.arange(len(subjects_final))
    width = 0.8 / len(models) # Dynamically size bars based on number of models
    
    # Define a colormap for the models
    colors_bar = plt.cm.Set2(np.linspace(0, 1, len(models)))
    
    for i, model in enumerate(models):
        # Extract values for this model across all subjects
        model_data = []
        for subj in subjects_final:
            val = df_final[(df_final['subject'] == subj) & (df_final['model'] == model)]
            if not val.empty:
                # Use mean in case there are multiple runs for the exact same subject+model pairing
                model_data.append(val[metric_col].mean())
            else:
                model_data.append(0)
                
        # Offset the bars so they group together side-by-side
        offset = (i - len(models)/2 + 0.5) * width
        plt.bar(x + offset, model_data, width, label=model, color=colors_bar[i], edgecolor='black')

    plt.xlabel('Cache Architecture (Subject)', fontsize=12)
    plt.ylabel(ylabel, fontsize=12)
    plt.title(title, fontsize=14)
    plt.xticks(x, subjects_final, rotation=15)
    
    plt.grid(axis='y', linestyle='--', alpha=0.7)
    
    plt.tight_layout()
    plt.savefig(f'plots/{filename}', dpi=300)
    plt.close()

# Generate the three bar plots
plot_grouped_bar('hit_rate', 'Final Hit Rate by Model and Subject', 'Hit Rate', 'final-hitrate-bar.png')
plot_grouped_bar('throughput', 'Final Throughput by Model and Subject', 'Operations / Second', 'final-throughput-bar.png')
plot_grouped_bar('metadata', 'Final Metadata Overhead by Model and Subject', 'Metadata Size (Bytes)', 'final-metadata-bar.png')

print("Successfully generated 3 grouped bar charts in the 'plots/' directory.")