import type { ModelProfile } from "../../types/provider";

interface ProviderModelListProps {
  models: ModelProfile[];
}

export function ProviderModelList({ models }: ProviderModelListProps) {
  return (
    <section className="provider-model-list" id="models" aria-labelledby="provider-models-heading">
      <div className="provider-panel-heading">
        <div>
          <p className="eyebrow">Available routes</p>
          <h2 id="provider-models-heading">Models</h2>
        </div>
        <span className="provider-count">{models.length}</span>
      </div>
      {models.length > 0 ? (
        <ul>
          {models.map((model) => (
            <li key={`${model.provider_id}:${model.model_id}`}>
              <div>
                <strong>{model.display_name}</strong>
                <code>{model.model_id}</code>
              </div>
              <span className="provider-model-tier">{model.tier}</span>
            </li>
          ))}
        </ul>
      ) : (
        <p className="provider-no-models">No models reported.</p>
      )}
    </section>
  );
}
