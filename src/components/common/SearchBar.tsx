interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
}

export function SearchBar({ value, onChange }: SearchBarProps) {
  return (
    <label className="search-shell">
      <span className="search-label">Search packages</span>
      <input
        className="search-input"
        placeholder="Find by name, tag, or intent"
        value={value}
        onChange={(event) => {
          onChange(event.target.value);
        }}
      />
    </label>
  );
}
