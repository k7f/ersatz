use std::str::FromStr;
use wast::parser::{Parse, Peek, Cursor, Parser, ParseBuffer};
use crate::{Ersatz, Ground, Site, Reaction, Entity};

mod kw {
    wast::custom_keyword!(site);
    wast::custom_keyword!(trigger);
    wast::custom_keyword!(sequence);
    wast::custom_keyword!(entities);
    wast::custom_keyword!(choice);
}

impl FromStr for Ersatz {
    type Err = wast::Error;

    fn from_str(spec: &str) -> Result<Self, Self::Err> {
        let buf = ParseBuffer::new(spec)?;

        wast::parser::parse::<Ersatz>(&buf)
    }
}

impl<'a> Parse<'a> for Ersatz {
    fn parse(parser: Parser<'a>) -> wast::parser::Result<Self> {
        let mut sites = Vec::new();
        let mut tropes = Vec::new(); // global tropes

        while !parser.is_empty() {
            parser.parens(|p| {
                let mut l1 = p.lookahead1();

                if l1.peek::<SiteAst>() {
                    sites.push(p.parse::<SiteAst>()?);
                } else {
                    tropes.push(p.parse::<Trope>()?);
                }

                Ok(())
            })?;
        }

        let mut ground = Ground::new();

        for trope in tropes.iter_mut() {
            trope.compile(&mut ground);
        }

        for site in sites.iter_mut() {
            site.compile(&mut ground);
        }

        let ersatz = Ersatz::new().with_sites(sites.into_iter());

        Ok(ersatz)
    }
}

#[derive(Default, Debug)]
struct SiteAst<'a> {
    tropes:   Vec<Trope<'a>>,
    compiled: Option<Vec<Reaction>>,
}

impl<'a> SiteAst<'a> {
    fn compile(&mut self, ground: &mut Ground) {
        if self.compiled.is_none() {
            for trope in self.tropes.iter_mut() {
                trope.compile(ground);
            }

            let mut reactions = Vec::new();

            for trope in self.tropes.drain_filter(|t| match t {
                Trope::Trigger(_) | Trope::Sequence(_) => true,
                _ => false,
            }) {
                match trope {
                    Trope::Trigger(ast) => reactions.push(ast.into()),
                    Trope::Sequence(ast) => {
                        let rns: Vec<_> = ast.into();
                        reactions.extend(rns.into_iter());
                    }
                    _ => {}
                }
            }

            self.compiled = Some(reactions);
        }
    }
}

impl<'a> Parse<'a> for SiteAst<'a> {
    fn parse(parser: Parser<'a>) -> wast::parser::Result<Self> {
        parser.parse::<kw::site>()?;

        let mut tropes = Vec::new();

        while !parser.is_empty() {
            tropes.push(parser.parens(|p| p.parse::<Trope>())?);
        }

        Ok(SiteAst { tropes, ..Default::default() })
    }
}

impl Peek for SiteAst<'_> {
    fn peek(cursor: Cursor<'_>) -> bool {
        match cursor.keyword() {
            Some(("site", _)) => true,
            _ => false,
        }
    }

    fn display() -> &'static str {
        "a site"
    }
}

impl<'a> From<SiteAst<'a>> for Site {
    fn from(ast: SiteAst<'a>) -> Self {
        Site::new().with_reactions(ast.compiled.unwrap_or_default())
    }
}

#[derive(Debug)]
enum Trope<'a> {
    Trigger(TriggerAst<'a>),
    Sequence(SequenceAst<'a>),
    Entities(EntitiesAst<'a>),
    Choice(ChoiceAst<'a>),
}

impl<'a> Trope<'a> {
    fn compile(&mut self, ground: &mut Ground) {
        match self {
            Trope::Trigger(ast) => ast.compile(ground),
            Trope::Sequence(ast) => ast.compile(ground),
            _ => {}
        }
    }
}

impl<'a> Parse<'a> for Trope<'a> {
    fn parse(parser: Parser<'a>) -> wast::parser::Result<Self> {
        let mut l1 = parser.lookahead1();

        if l1.peek::<TriggerAst>() {
            Ok(Trope::Trigger(parser.parse()?))
        } else if l1.peek::<SequenceAst>() {
            Ok(Trope::Sequence(parser.parse()?))
        } else if l1.peek::<EntitiesAst>() {
            Ok(Trope::Entities(parser.parse()?))
        } else if l1.peek::<ChoiceAst>() {
            Ok(Trope::Choice(parser.parse()?))
        } else {
            Err(l1.error())
        }
    }
}

#[derive(Default, Debug)]
struct TriggerAst<'a> {
    id:       Option<wast::Id<'a>>,
    entities: Vec<EntityToken<'a>>,
    compiled: Option<Reaction>,
}

impl<'a> TriggerAst<'a> {
    fn compile(&mut self, ground: &mut Ground) {
        if self.compiled.is_none() {
            let reaction = Reaction::new().with_products(self.entities.drain(..));

            self.compiled = Some(reaction);
        }
    }
}

impl<'a> Parse<'a> for TriggerAst<'a> {
    fn parse(parser: Parser<'a>) -> wast::parser::Result<Self> {
        parser.parse::<kw::trigger>()?;

        let id = if parser.lookahead1().peek::<wast::LParen>() {
            None
        } else {
            Some(parser.parse::<wast::Id>()?)
        };

        let entities = parser.parens(|p| {
            let mut ents = Vec::new();

            while !p.is_empty() {
                ents.push(parser.parse()?);
            }

            Ok(ents)
        })?;

        Ok(TriggerAst { id, entities, ..Default::default() })
    }
}

impl Peek for TriggerAst<'_> {
    fn peek(cursor: Cursor<'_>) -> bool {
        match cursor.keyword() {
            Some(("trigger", _)) => true,
            _ => false,
        }
    }

    fn display() -> &'static str {
        "a trigger"
    }
}

impl<'a> From<TriggerAst<'a>> for Reaction {
    fn from(ast: TriggerAst<'a>) -> Self {
        if ast.compiled.is_none() {
            Reaction::new().with_products(ast.entities.into_iter())
        } else {
            ast.compiled.unwrap()
        }
    }
}

#[derive(Default, Debug)]
struct SequenceAst<'a> {
    id:       Option<wast::Id<'a>>,
    terms:    Vec<Vec<EntityToken<'a>>>,
    compiled: Option<Vec<Reaction>>,
}

impl<'a> SequenceAst<'a> {
    fn compile(&mut self, ground: &mut Ground) {
        if self.compiled.is_none() {
            let mut reactions = Vec::new();
            let mut rn = Reaction::new();

            for term in self.terms.drain(..) {
                rn.p.extend(term.into_iter().map(Into::into));
                reactions.push(rn.clone());
                rn.r.clear();
                rn.r.extend_numbers(rn.p.drain_numbers());
                rn.r.extend_names(rn.p.drain_names());
            }

            self.compiled = Some(reactions);
        }
    }
}

impl<'a> Parse<'a> for SequenceAst<'a> {
    fn parse(parser: Parser<'a>) -> wast::parser::Result<Self> {
        parser.parse::<kw::sequence>()?;

        let id = if parser.lookahead1().peek::<wast::LParen>() {
            None
        } else {
            Some(parser.parse::<wast::Id>()?)
        };

        let mut terms = Vec::new();

        while !parser.is_empty() {
            terms.push(parser.parens(|p| {
                let mut ents = Vec::new();

                while !p.is_empty() {
                    ents.push(parser.parse()?);
                }

                Ok(ents)
            })?);
        }

        Ok(SequenceAst { id, terms, ..Default::default() })
    }
}

impl Peek for SequenceAst<'_> {
    fn peek(cursor: Cursor<'_>) -> bool {
        match cursor.keyword() {
            Some(("sequence", _)) => true,
            _ => false,
        }
    }

    fn display() -> &'static str {
        "a sequence"
    }
}

impl<'a> From<SequenceAst<'a>> for Vec<Reaction> {
    fn from(ast: SequenceAst<'a>) -> Self {
        if ast.compiled.is_none() {
            let mut reactions = Vec::new();
            let mut rn = Reaction::new();

            for term in ast.terms {
                rn.p.extend(term.into_iter().map(Into::into));
                reactions.push(rn.clone());
                rn.r.clear();
                rn.r.extend_numbers(rn.p.drain_numbers());
                rn.r.extend_names(rn.p.drain_names());
            }

            reactions
        } else {
            ast.compiled.unwrap()
        }
    }
}

#[derive(Default, Debug)]
struct EntitiesAst<'a> {
    id:       Option<wast::Id<'a>>,
    entities: Vec<EntityToken<'a>>,
}

impl<'a> Parse<'a> for EntitiesAst<'a> {
    fn parse(parser: Parser<'a>) -> wast::parser::Result<Self> {
        parser.parse::<kw::entities>()?;

        let id = if parser.lookahead1().peek::<wast::LParen>() {
            None
        } else {
            Some(parser.parse::<wast::Id>()?)
        };

        let entities = parser.parens(|p| {
            let mut ents = Vec::new();

            while !p.is_empty() {
                ents.push(parser.parse()?);
            }

            Ok(ents)
        })?;

        Ok(EntitiesAst { id, entities })
    }
}

impl Peek for EntitiesAst<'_> {
    fn peek(cursor: Cursor<'_>) -> bool {
        match cursor.keyword() {
            Some(("entities", _)) => true,
            _ => false,
        }
    }

    fn display() -> &'static str {
        "entities"
    }
}

#[derive(Default, Debug)]
struct ChoiceAst<'a> {
    id:    Option<wast::Id<'a>>,
    cards: Vec<u32>,
    base:  Vec<EntityToken<'a>>,
}

impl<'a> Parse<'a> for ChoiceAst<'a> {
    fn parse(parser: Parser<'a>) -> wast::parser::Result<Self> {
        parser.parse::<kw::choice>()?;

        let id = if parser.lookahead1().peek::<wast::LParen>() {
            None
        } else {
            Some(parser.parse::<wast::Id>()?)
        };

        let cards = parser.parens(|p| {
            let mut cards = Vec::new();

            while !p.is_empty() {
                cards.push(parser.parse()?);
            }

            Ok(cards)
        })?;

        let base = parser.parens(|p| {
            let mut ents = Vec::new();

            while !p.is_empty() {
                ents.push(parser.parse()?);
            }

            Ok(ents)
        })?;

        Ok(ChoiceAst { id, cards, base })
    }
}

impl Peek for ChoiceAst<'_> {
    fn peek(cursor: Cursor<'_>) -> bool {
        match cursor.keyword() {
            Some(("choice", _)) => true,
            _ => false,
        }
    }

    fn display() -> &'static str {
        "a choice"
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EntityToken<'a> {
    Number(u32),
    Name(NameLiteral<'a>),
    Identifier(wast::Id<'a>), // an identifier of a _set_ of entities defined elsewhere
}

impl<'a> Parse<'a> for EntityToken<'a> {
    fn parse(parser: Parser<'a>) -> wast::parser::Result<Self> {
        let mut l1 = parser.lookahead1();
        if l1.peek::<wast::Id>() {
            Ok(EntityToken::Identifier(parser.parse()?))
        } else if l1.peek::<NameLiteral>() {
            Ok(EntityToken::Name(parser.parse()?))
        } else if l1.peek::<u32>() {
            Ok(EntityToken::Number(parser.parse()?))
        } else {
            Err(l1.error())
        }
    }
}

impl Peek for EntityToken<'_> {
    fn peek(cursor: Cursor<'_>) -> bool {
        u32::peek(cursor) || wast::Id::peek(cursor) || NameLiteral::peek(cursor)
    }

    fn display() -> &'static str {
        "an entity"
    }
}

impl<'a> From<EntityToken<'a>> for Entity {
    fn from(token: EntityToken<'a>) -> Self {
        use EntityToken::*;

        match token {
            Number(num) => Entity::Number(num),
            Name(name) => Entity::Name(name.0.into()),
            Identifier(id) => Entity::Identifier(id.name().into()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct NameLiteral<'a>(&'a str);

impl<'a> Parse<'a> for NameLiteral<'a> {
    fn parse(parser: Parser<'a>) -> wast::parser::Result<Self> {
        let name = parser.step(|c| c.keyword().ok_or_else(|| parser.error("not a name")))?;

        Ok(NameLiteral(name))
    }
}

impl Peek for NameLiteral<'_> {
    fn peek(cursor: Cursor<'_>) -> bool {
        cursor.keyword().is_some()
    }

    fn display() -> &'static str {
        "a name"
    }
}
