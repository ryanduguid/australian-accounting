// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

// The tool calls compared with the native server: the 32 conformance cases,
// the no-argument tools and the data freshness report for each served tool.
export function requests(cases) {
  const served = ['calc_payday_super_deadline', 'get_div7a_benchmark_rate', 'review_div7a_loan', 'calculate_tax_worksheet'];
  return [
    ...cases.map(c => ({ name: c.id, tool: c.tool, arguments: c.arguments })),
    { name: 'refuse_div7a', tool: 'refuse_div7a', arguments: {} },
    ...served.map(tool => ({ name: `freshness_${tool}`, tool: 'get_data_freshness', arguments: { tool } })),
    // The native server also reports engines the connector does not link, so
    // this one is compared after filtering to the served tools' datasets.
    { name: 'freshness_all', tool: 'get_data_freshness', arguments: {} },
    { name: 'freshness_as_at', tool: 'get_data_freshness', arguments: { tool: 'get_div7a_benchmark_rate', as_at: '2026-11-14' } },
  ];
}
