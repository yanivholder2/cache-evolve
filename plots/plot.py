import json
import os

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

# Fitness dimensions carried per variant. A value of -1 means the dimension
# was not calculated for that variant and must not be plotted.
MISSING = -1

def generate_plots(data_file, out_dir):
    # Create the figures directory if it doesn't exist
    os.makedirs(out_dir, exist_ok=True)

    # 1. Load the data
    with open(data_file, 'r') as f:
        data = json.load(f)

    # ==========================================
    # PART 1: LINE CHARTS (Evolution over Cycles)
    # ==========================================

    # Flatten the winning variant of each cycle into a DataFrame. The evolution
    # lines track the cycle frontier, so only winners (plus the baseline) count.
    rows = []
    for run in data['runs']:
        for v in run['variants']:
            if not (v['is-winner'] or v['is-baseline']):
                continue
            rows.append({
                'subject': v['subject-name'],
                'cycle': v['cycle'],
                'hit_rate': v['hit-rate'],
                'metadata_size': v['metadata-size'],
                'per_op_time': v['per-op-time'],
                'weighted_score': v.get('weighted_score', v.get('weighted-score', MISSING)),
            })

    df = pd.DataFrame(rows)

    # Treat the -1 sentinel as missing so it is excluded from means and plots.
    for col in ('hit_rate', 'metadata_size', 'per_op_time', 'weighted_score'):
        df[col] = df[col].replace(MISSING, np.nan)

    # Group by subject and cycle, averaging across co-winners of a cycle.
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

    def plot_evolution(metric_col, title, ylabel, filename):
        plt.figure(figsize=(6, 4))
        for subj in subjects:
            subj_data = df_agg[df_agg['subject'] == subj].sort_values('cycle')
            # Drop cycles where this metric was never calculated.
            subj_data = subj_data.dropna(subset=[metric_col])
            if subj_data.empty:
                continue
            plt.plot(subj_data['cycle'], subj_data[metric_col],
                     color=style_map[subj]['color'], marker=style_map[subj]['marker'], linewidth=2)
        plt.xlabel('Cycles', fontsize=12)
        plt.ylabel(ylabel, fontsize=12)
        plt.title(title, fontsize=14)
        plt.grid(True, linestyle='--', alpha=0.7)
        plt.tight_layout()
        plt.savefig(os.path.join(out_dir, filename), dpi=300)
        plt.close()

    plot_evolution('hit_rate', 'Evolution of Hit Rate', 'Hit Rate', 'hitrate-vs-cycles.png')
    plot_evolution('metadata_size', 'Evolution of Metadata Overhead', 'Metadata Size (bytes)', 'metadata-vs-cycles.png')
    plot_evolution('per_op_time', 'Evolution of Per-op Time', 'Per-op Time', 'per-op-time-vs-cycles.png')
    plot_evolution('weighted_score', 'Evolution of Weighted Score', 'Weighted Score', 'weighted-score-vs-cycles.png')

    # Standalone Legend
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

    fig_leg.savefig(os.path.join(out_dir, 'legend.png'), bbox_inches='tight', dpi=300)
    plt.close()

    print(f"Successfully generated 4 cycle evolution plots and 1 standalone legend in the '{out_dir}/' directory.")

    # ==========================================
    # PART 2: GROUPED BAR CHARTS (Final Performance by Agent Tool)
    # ==========================================

    # Extract the final-cycle winner for each run. Co-winners of the last cycle
    # are averaged; -1 dimensions stay missing and are skipped by the bars.
    final_variants = []
    for run in data['runs']:
        agent_tool = run['setup']['ai-agent-tool']
        winners = [v for v in run['variants'] if v['is-winner']]
        if not winners:
            continue
        last_cycle = max(v['cycle'] for v in winners)
        last_winners = [v for v in winners if v['cycle'] == last_cycle]

        def mean_metric(key, alt_key=None):
            vals = []
            for v in last_winners:
                val = v.get(key, v.get(alt_key, MISSING)) if alt_key else v[key]
                if val != MISSING:
                    vals.append(val)
            return np.mean(vals) if vals else np.nan

        final_variants.append({
            'subject': last_winners[0]['subject-name'],
            'agent_tool': agent_tool,
            'hit_rate': mean_metric('hit-rate'),
            'metadata_size': mean_metric('metadata-size'),
            'per_op_time': mean_metric('per-op-time'),
            'weighted_score': mean_metric('weighted_score', 'weighted-score'),
        })

    df_final = pd.DataFrame(final_variants)

    # Setup plotting function for grouped bars
    def plot_grouped_bar(metric_col, title, ylabel, filename):
        plt.figure(figsize=(8, 5))

        subjects_final = df_final['subject'].unique()
        agent_tools = df_final['agent_tool'].unique()

        x = np.arange(len(subjects_final))
        width = 0.8 / len(agent_tools)  # Dynamically size bars based on number of agent tools

        # Define a colormap for the agent tools
        colors_bar = plt.cm.Set2(np.linspace(0, 1, len(agent_tools)))

        for i, agent_tool in enumerate(agent_tools):
            # Extract values for this agent tool across all subjects
            tool_data = []
            for subj in subjects_final:
                val = df_final[(df_final['subject'] == subj) & (df_final['agent_tool'] == agent_tool)]
                metric = val[metric_col].mean() if not val.empty else np.nan
                # A NaN bar (dimension never calculated) renders as no bar.
                tool_data.append(0 if np.isnan(metric) else metric)

            # Offset the bars so they group together side-by-side
            offset = (i - len(agent_tools) / 2 + 0.5) * width
            plt.bar(x + offset, tool_data, width, label=agent_tool, color=colors_bar[i], edgecolor='black')

        plt.xlabel('Cache Architecture (Subject)', fontsize=12)
        plt.ylabel(ylabel, fontsize=12)
        plt.title(title, fontsize=14)
        plt.xticks(x, subjects_final, rotation=15)

        plt.grid(axis='y', linestyle='--', alpha=0.7)

        plt.tight_layout()
        plt.savefig(os.path.join(out_dir, filename), dpi=300)
        plt.close()

    # Generate the four bar plots
    plot_grouped_bar('hit_rate', 'Final Hit Rate by Agent Tool and Subject', 'Hit Rate', 'final-hitrate-bar.png')
    plot_grouped_bar('metadata_size', 'Final Metadata Overhead by Agent Tool and Subject', 'Metadata Size (bytes)', 'final-metadata-bar.png')
    plot_grouped_bar('per_op_time', 'Final Per-op Time by Agent Tool and Subject', 'Per-op Time', 'final-per-op-time-bar.png')
    plot_grouped_bar('weighted_score', 'Final Weighted Score by Agent Tool and Subject', 'Weighted Score', 'final-weighted-score-bar.png')

    print(f"Successfully generated 4 grouped bar charts in the '{out_dir}/' directory.")

# Run plot generation on both files, dispatching output into their respective folders.
generate_plots('plots/paper-plots/data_full_fitness_vector.json', 'plots/paper-plots/full_fitness_vector')
generate_plots('plots/paper-plots/data_only_hit_rate.json', 'plots/paper-plots/only_hit_rate')
