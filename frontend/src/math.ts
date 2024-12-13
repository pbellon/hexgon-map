type Scale = (input: number) => number;

type LinearScaleParams = {
  domain: [number, number];
  range: [number, number];
};

export const linearScale = ({ domain, range }: LinearScaleParams): Scale => {
  const [minDomain, maxDomain] = domain;
  const [minRange, maxRange] = range;

  return (input: number) =>
    minRange +
    ((input - minDomain) * (maxRange - minRange)) / (maxDomain - minDomain);
};
