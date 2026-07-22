import json
import os

import matplotlib.patches as mpatches
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
import numpy as np
import pandas as pd

# Fitness dimensions carried per variant. A value of -1 means the dimension
# was not calculated for that variant and must not be plotted.
MISSING = -1

def get_model_from_run(run):
    subj = run['setup']['subject-name']
    run_name = run['setup'].get('run', '')
    if run_name.startswith(subj + '_'):
        suffix = run_name[len(subj) + 1:]
        # Extract the model part by taking everything before the first year string like _202
        model = suffix.split('_202')[0] if '_202' in suffix else suffix.split('_')[0]
        return model
    return run['setup'].get('ai-agent-tool', 'unknown')

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
        plt.figure()
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

    # Consistent color per model (agent tool), shared across the wasted-cost
    # and attempts-count bar charts so a color always means the same model,
    # backed by one standalone legend instead of a per-chart legend.
    all_models = sorted(set(get_model_from_run(r) for r in data['runs']))
    model_colors = plt.cm.Set2(np.linspace(0, 1, len(all_models)))
    model_style_map = {model: model_colors[i] for i, model in enumerate(all_models)}

    # ==========================================
    # PART 2: GROUPED BAR CHARTS (Final Performance by Agent Tool)
    # ==========================================

    # Extract the final-cycle winner for each run. Co-winners of the last cycle
    # are averaged; -1 dimensions stay missing and are skipped by the bars.
    final_variants = []
    for run in data['runs']:
        agent_tool = get_model_from_run(run)
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
        plt.figure()

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

    # ==========================================
    # PART 3: PARETO FRONTIER PLOTS (Best Score vs Cost/Runtime)
    # ==========================================
    # Restricted to s3_fifo and lru_cache to keep the frontier legible; color
    # encodes model (matching model_style_map elsewhere), marker encodes subject.

    PARETO_SUBJECTS = ('s3_fifo', 'lru_cache')
    all_markers = ['o', 's', '^', 'D', 'v', 'p', '*', 'X', '<', '>']
    subject_marker_map = {subj: all_markers[i % len(all_markers)] for i, subj in enumerate(PARETO_SUBJECTS)}
    # s3_fifo dashed, lru_cache solid, so the two policies are readable even
    # in grayscale/without markers.
    subject_linestyle_map = {'s3_fifo': '--', 'lru_cache': '-'}

    frontier_rows = []
    for run_idx, run in enumerate(data['runs']):
        subject_name = run['setup']['subject-name']
        if subject_name not in PARETO_SUBJECTS:
            continue

        cumulative_cost = 0.0
        cumulative_runtime = 0.0
        # Start at 0 for baseline. Negative scores are ignored for champions.
        best_score = 0.0
        agent_tool = get_model_from_run(run)

        # Add the baseline/start point
        frontier_rows.append({
            'run_idx': run_idx,
            'subject': subject_name,
            'agent_tool': agent_tool,
            'cumulative_cost': 0.0,
            'cumulative_runtime': 0.0,
            'score': 0.0
        })

        for v in run['variants']:
            cost = v.get('cost-usd', MISSING)
            if cost != MISSING and cost != -1:
                cumulative_cost += cost

            runtime = v.get('total-seconds-to-generate', MISSING)
            if runtime != MISSING and runtime != -1:
                cumulative_runtime += runtime

            score = v.get('weighted_score', v.get('weighted-score', MISSING))
            if score != MISSING and score != -1:
                # Only keep point if it's strictly greater than the highest score seen so far
                if score > best_score:
                    best_score = score
                    frontier_rows.append({
                        'run_idx': run_idx,
                        'subject': v['subject-name'],
                        'agent_tool': agent_tool,
                        'cumulative_cost': cumulative_cost,
                        'cumulative_runtime': cumulative_runtime,
                        'score': best_score
                    })

    df_frontier = pd.DataFrame(frontier_rows)
    if not df_frontier.empty:
        # Plot Cost vs Score (Pareto Frontier)
        plt.figure()
        for subj in PARETO_SUBJECTS:
            subj_data = df_frontier[df_frontier['subject'] == subj]
            if subj_data.empty:
                continue
            for run_idx in subj_data['run_idx'].unique():
                run_data = subj_data[subj_data['run_idx'] == run_idx].sort_values('cumulative_cost')
                agent = run_data['agent_tool'].iloc[0]
                plt.step(run_data['cumulative_cost'], run_data['score'], where='post',
                         color=model_style_map[agent], marker=subject_marker_map[subj],
                         linestyle=subject_linestyle_map[subj],
                         label=f"{subj} ({agent})", alpha=0.7, linewidth=2)

        plt.xlabel('Total Cost (USD)', fontsize=12)
        plt.ylabel('Best Score', fontsize=12)
        plt.grid(True, linestyle='--', alpha=0.7)
        # Legend shows only the subject/policy shape and line style, filled
        # white — color already encodes model via the shared model-legend.png.
        shape_handles = [
            Line2D([0], [0], marker=subject_marker_map[subj], color='black',
                   linestyle=subject_linestyle_map[subj],
                   markersize=8, markerfacecolor='white', label=subj)
            for subj in PARETO_SUBJECTS
        ]
        plt.legend(handles=shape_handles, fontsize=10, loc='lower right')
        plt.tight_layout()
        plt.savefig(os.path.join(out_dir, 'cost-vs-score-pareto.png'), dpi=300, bbox_inches='tight')
        plt.close()
        print(f"Successfully generated cost vs capability pareto plot in the '{out_dir}/' directory.")

        # Plot Runtime vs Score (Pareto Frontier)
        plt.figure()
        for subj in PARETO_SUBJECTS:
            subj_data = df_frontier[df_frontier['subject'] == subj]
            if subj_data.empty:
                continue
            for run_idx in subj_data['run_idx'].unique():
                run_data = subj_data[subj_data['run_idx'] == run_idx].sort_values('cumulative_runtime')
                agent = run_data['agent_tool'].iloc[0]
                plt.step(run_data['cumulative_runtime'], run_data['score'], where='post',
                         color=model_style_map[agent], marker=subject_marker_map[subj],
                         linestyle=subject_linestyle_map[subj],
                         label=f"{subj} ({agent})", alpha=0.7, linewidth=2)

        plt.xlabel('Total Runtime (Seconds)', fontsize=12)
        plt.ylabel('Best Score', fontsize=12)
        plt.grid(True, linestyle='--', alpha=0.7)
        # Legend shows only the subject/policy shape and line style, filled
        # white — color already encodes model via the shared model-legend.png.
        shape_handles = [
            Line2D([0], [0], marker=subject_marker_map[subj], color='black',
                   linestyle=subject_linestyle_map[subj],
                   markersize=8, markerfacecolor='white', label=subj)
            for subj in PARETO_SUBJECTS
        ]
        plt.legend(handles=shape_handles, fontsize=10, loc='lower right')
        plt.tight_layout()
        plt.savefig(os.path.join(out_dir, 'runtime-vs-score-pareto.png'), dpi=300, bbox_inches='tight')
        plt.close()
        print(f"Successfully generated runtime vs capability pareto plot in the '{out_dir}/' directory.")

    # ==========================================
    # PART 4: WASTED COST (Grouped Stacked Bar Chart: Productive vs Wasted)
    # ==========================================
    print("Generating wasted cost bar plot...")

    cost_rows = []
    for run in data['runs']:
        setup = run.get('setup', {})
        agent_tool = get_model_from_run(run)
        subject_name = setup.get('subject-name', 'unknown')
        productive_cost = setup.get('productive-cost-usd', 0.0)
        sunk_cost = setup.get('sunk-cost-usd', 0.0)

        if productive_cost == MISSING or productive_cost is None: productive_cost = 0.0
        if sunk_cost == MISSING or sunk_cost is None: sunk_cost = 0.0

        cost_rows.append({
            'agent_tool': agent_tool,
            'subject': subject_name,
            'productive_cost': productive_cost,
            'sunk_cost': sunk_cost,
        })

    df_cost = pd.DataFrame(cost_rows)
    if not df_cost.empty:
        plt.figure()

        subjects_cost = sorted(df_cost['subject'].unique())
        agent_tools_cost = sorted(df_cost['agent_tool'].unique())

        x = np.arange(len(subjects_cost))
        width = 0.8 / len(agent_tools_cost)
        max_total = 0.0

        for i, agent_tool in enumerate(agent_tools_cost):
            productive_vals = []
            sunk_vals = []
            for subj in subjects_cost:
                match = df_cost[(df_cost['subject'] == subj) & (df_cost['agent_tool'] == agent_tool)]
                if not match.empty:
                    productive_vals.append(match['productive_cost'].mean())
                    sunk_vals.append(match['sunk_cost'].mean())
                else:
                    productive_vals.append(0.0)
                    sunk_vals.append(0.0)

            productive_vals = np.array(productive_vals)
            sunk_vals = np.array(sunk_vals)
            max_total = max(max_total, (productive_vals + sunk_vals).max())

            offset = (i - len(agent_tools_cost) / 2 + 0.5) * width
            color = model_style_map[agent_tool]
            # Bottom segment: productive cost, solid. Top segment: wasted (sunk)
            # cost, same model color but hatched so the split reads at a glance.
            plt.bar(x + offset, productive_vals, width, color=color, edgecolor='black')
            plt.bar(x + offset, sunk_vals, width, bottom=productive_vals,
                    color=color, edgecolor='black', hatch='//', alpha=0.6)

            for xi, p, s in zip(x + offset, productive_vals, sunk_vals):
                total = p + s
                if total > 1e-9:
                    pct = s / total * 100
                    plt.text(xi, total + total * 0.05 + 0.05, f"{pct:.0f}%",
                             ha='center', va='bottom', fontsize=8, rotation=90)

        # Headroom above the tallest bar so the rotated percentage labels
        # (which sit above the bar) aren't clipped by the plot's top edge.
        plt.ylim(0, max_total * 1.3)

        plt.xlabel('Cache Architecture (Subject)', fontsize=12)
        plt.ylabel('Total Cost (USD)', fontsize=12)
        plt.xticks(x, subjects_cost, rotation=15)

        shading_handles = [
            mpatches.Patch(facecolor='lightgray', edgecolor='black', label='Productive Cost'),
            mpatches.Patch(facecolor='lightgray', edgecolor='black', hatch='//', alpha=0.6, label='Wasted Cost'),
        ]
        plt.legend(handles=shading_handles, fontsize=9, loc='upper left')

        plt.grid(axis='y', linestyle='--', alpha=0.7)

        plt.tight_layout()
        plt.savefig(os.path.join(out_dir, 'wasted-cost-bar.png'), dpi=300)
        plt.close()
        print(f"Successfully generated wasted cost bar plot in the '{out_dir}/' directory.")

    # ==========================================
    # PART 4b: WASTED RUNTIME (Grouped Stacked Bar Chart: Productive vs Wasted)
    # ==========================================
    print("Generating wasted runtime bar plot...")

    runtime_rows = []
    for run in data['runs']:
        setup = run.get('setup', {})
        agent_tool = get_model_from_run(run)
        subject_name = setup.get('subject-name', 'unknown')
        productive_seconds = setup.get('productive-runtime-seconds', 0.0)
        sunk_seconds = setup.get('sunk-runtime-seconds', 0.0)

        if productive_seconds == MISSING or productive_seconds is None: productive_seconds = 0.0
        if sunk_seconds == MISSING or sunk_seconds is None: sunk_seconds = 0.0

        runtime_rows.append({
            'agent_tool': agent_tool,
            'subject': subject_name,
            'productive_seconds': productive_seconds,
            'sunk_seconds': sunk_seconds,
        })

    df_runtime = pd.DataFrame(runtime_rows)
    if not df_runtime.empty:
        plt.figure()

        subjects_runtime = sorted(df_runtime['subject'].unique())
        agent_tools_runtime = sorted(df_runtime['agent_tool'].unique())

        x = np.arange(len(subjects_runtime))
        width = 0.8 / len(agent_tools_runtime)
        max_total = 0.0

        for i, agent_tool in enumerate(agent_tools_runtime):
            productive_vals = []
            sunk_vals = []
            for subj in subjects_runtime:
                match = df_runtime[(df_runtime['subject'] == subj) & (df_runtime['agent_tool'] == agent_tool)]
                if not match.empty:
                    productive_vals.append(match['productive_seconds'].mean())
                    sunk_vals.append(match['sunk_seconds'].mean())
                else:
                    productive_vals.append(0.0)
                    sunk_vals.append(0.0)

            productive_vals = np.array(productive_vals)
            sunk_vals = np.array(sunk_vals)
            max_total = max(max_total, (productive_vals + sunk_vals).max())

            offset = (i - len(agent_tools_runtime) / 2 + 0.5) * width
            color = model_style_map[agent_tool]
            # Bottom segment: productive runtime, solid. Top segment: wasted (sunk)
            # runtime, same model color but hatched so the split reads at a glance.
            plt.bar(x + offset, productive_vals, width, color=color, edgecolor='black')
            plt.bar(x + offset, sunk_vals, width, bottom=productive_vals,
                    color=color, edgecolor='black', hatch='//', alpha=0.6)

            for xi, p, s in zip(x + offset, productive_vals, sunk_vals):
                total = p + s
                if total > 1e-9:
                    pct = s / total * 100
                    plt.text(xi, total + total * 0.05 + 0.05, f"{pct:.0f}%",
                             ha='center', va='bottom', fontsize=8, rotation=90)

        # Headroom above the tallest bar so the rotated percentage labels
        # (which sit above the bar) aren't clipped by the plot's top edge.
        plt.ylim(0, max_total * 1.3)

        plt.xlabel('Cache Architecture (Subject)', fontsize=12)
        plt.ylabel('Total Runtime (Seconds)', fontsize=12)
        plt.xticks(x, subjects_runtime, rotation=15)

        shading_handles = [
            mpatches.Patch(facecolor='lightgray', edgecolor='black', label='Productive Runtime'),
            mpatches.Patch(facecolor='lightgray', edgecolor='black', hatch='//', alpha=0.6, label='Wasted Runtime'),
        ]
        plt.legend(handles=shading_handles, fontsize=9, loc='upper left')

        plt.grid(axis='y', linestyle='--', alpha=0.7)

        plt.tight_layout()
        plt.savefig(os.path.join(out_dir, 'wasted-runtime-bar.png'), dpi=300)
        plt.close()
        print(f"Successfully generated wasted runtime bar plot in the '{out_dir}/' directory.")

    # ==========================================
    # PART 5: FAILED VARIANTS (Grouped Bar Chart with Error Bars)
    # ==========================================
    print("Generating count attempts bar plot...")
    failed_rows = []
    for run in data['runs']:
        setup = run.get('setup', {})
        agent_tool = get_model_from_run(run)
        subject_name = setup.get('subject-name', 'unknown')
        variants = run.get('variants', [])
        
        # Flatten variants so each attempt is its own row
        for v in variants:
            attempts = v.get('count-attempts', MISSING)
            if attempts != MISSING and attempts != -1:
                failed_rows.append({
                    'agent_tool': agent_tool,
                    'subject': subject_name,
                    'count_attempts': attempts
                })
        
    df_failed = pd.DataFrame(failed_rows)
    if not df_failed.empty:
        plt.figure()

        subjects_failed = sorted(df_failed['subject'].unique())
        agent_tools = sorted(df_failed['agent_tool'].unique())

        x = np.arange(len(subjects_failed))
        width = 0.8 / len(agent_tools)

        for i, agent_tool in enumerate(agent_tools):
            tool_means = []
            tool_stds = []
            for subj in subjects_failed:
                match = df_failed[(df_failed['subject'] == subj) & (df_failed['agent_tool'] == agent_tool)]
                if not match.empty:
                    val_mean = match['count_attempts'].mean()
                    val_std = match['count_attempts'].std()
                    # Pandas std() returns NaN if there's only 1 item. Replace with 0.0
                    if pd.isna(val_std):
                        val_std = 0.0
                else:
                    val_mean = 0.0
                    val_std = 0.0
                    
                tool_means.append(val_mean)
                tool_stds.append(val_std)

            offset = (i - len(agent_tools) / 2 + 0.5) * width
            # Added yerr=tool_stds and capsize=4 for the error bars
            plt.bar(x + offset, tool_means, width, yerr=tool_stds, label=agent_tool,
                    color=model_style_map[agent_tool], edgecolor='black', capsize=4)

        plt.xlabel('Cache Architecture (Subject)', fontsize=12)
        plt.ylabel('Count of Attempts (Mean ± StdDev)', fontsize=12)
        plt.xticks(x, subjects_failed, rotation=15)

        plt.grid(axis='y', linestyle='--', alpha=0.7)

        plt.tight_layout()
        plt.savefig(os.path.join(out_dir, 'count-attempts-bar.png'), dpi=300)
        plt.close()
        print(f"Successfully generated count attempts bar plot in the '{out_dir}/' directory.")

    # Standalone legend for the model color coding shared by the wasted-cost
    # and attempts-count bar charts above, instead of a legend on each chart.
    fig_leg = plt.figure(figsize=(10, 1.2))
    ax_leg = fig_leg.add_subplot(111)
    ax_leg.axis('off')

    legend_handles = [
        mpatches.Patch(facecolor=model_style_map[model], edgecolor='black', label=model)
        for model in all_models
    ]
    ax_leg.legend(handles=legend_handles, loc='center', ncol=len(all_models),
                  frameon=False, fontsize=12)

    fig_leg.savefig(os.path.join(out_dir, 'model-legend.png'), bbox_inches='tight', dpi=300)
    plt.close()
    print(f"Successfully generated model legend in the '{out_dir}/' directory.")

# Run plot generation on both files, dispatching output into their respective folders.
generate_plots('plots/paper-plots/data_full_fitness_vector.json', 'plots/paper-plots/full_fitness_vector')
generate_plots('plots/paper-plots/data_only_hit_rate.json', 'plots/paper-plots/only_hit_rate')
