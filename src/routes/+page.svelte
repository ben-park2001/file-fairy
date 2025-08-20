<script lang="ts">
  import MainLayout from "$lib/components/templates/main-layout.svelte";
  import type { SearchResult } from "$lib/types";

  let searchResults = $state<SearchResult[]>([]);
  let isSearching = $state(false);
  let hasSearched = $state(false);
  let lastSearchQuery = $state("");

  const handleSearch = async (query: string) => {
    if (!query.trim()) {
      searchResults = [];
      isSearching = false;
      hasSearched = false;
      lastSearchQuery = "";
      return;
    }

    hasSearched = true;
    lastSearchQuery = query;
    isSearching = true;
    // Simulate search API call
    setTimeout(() => {
      const mockResults: SearchResult[] = [
        {
          id: 1,
          filename: "Q4-Financial-Report.pdf",
          path: "/Documents/Reports",
          snippet: "Financial performance for Q4 2024 showing 15% growth...",
          type: "pdf",
        },
        {
          id: 2,
          filename: "budget-analysis.xlsx",
          path: "/Documents/Finance",
          snippet: "Budget allocation and expense analysis for fiscal year...",
          type: "excel",
        },
        {
          id: 3,
          filename: "meeting-notes.txt",
          path: "/Documents/Notes",
          snippet:
            "Team meeting notes discussing project roadmap and milestones...",
          type: "text",
        },
      ];

      searchResults = mockResults.filter(
        (result) =>
          result.filename.toLowerCase().includes(query.toLowerCase()) ||
          result.snippet.toLowerCase().includes(query.toLowerCase())
      );

      isSearching = false;
    }, 800);
  };

  const handleOrganize = () => {
    console.log("Organizing files...");
    // TODO: Implement file organization logic
  };
</script>

<MainLayout
  {searchResults}
  {isSearching}
  {hasSearched}
  {lastSearchQuery}
  onSearch={handleSearch}
  onOrganize={handleOrganize}
/>
